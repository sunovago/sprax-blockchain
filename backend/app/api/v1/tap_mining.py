import uuid
from datetime import UTC, datetime

from fastapi import APIRouter, Depends
from pydantic import BaseModel, Field
from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import settings
from app.core.dependencies import get_current_tap_miner, get_db
from app.core.logging import logger
from app.core.exceptions import (
    ALREADY_CHECKED_IN,
    CLAIM_COOLDOWN_ACTIVE,
    FAUCET_DISPATCH_FAILED,
    FORBIDDEN,
    INSUFFICIENT_POINTS,
    INVALID_ADDRESS,
    INVALID_REQUEST,
    NO_PAYOUT_ADDRESS,
    NOT_FOUND,
    RATE_LIMITED,
    TAP_MINING_DISABLED,
    TASK_ALREADY_CLAIMED,
    UNAUTHORIZED,
    SpraxError,
)
from app.core.response import ApiResponse
from app.core.security import create_access_token, verify_access_token
from app.core.telegram_auth import validate_init_data
from app.db.models.admin import FeatureFlag
from app.db.models.tap_mining import TapMiner, TapMinerActivity, TapMinerClaim, TapMinerTaskCompletion, TapMiningTask
from app.db.redis import redis_get, redis_set
from app.modules.tap_mining.checkin import checkin_reward, next_streak
from app.modules.tap_mining.claim import (
    calculate_payout,
    dispatch_faucet_payout,
    get_cooldown_remaining,
    validate_recipient_address,
)
from app.modules.tap_mining.energy import compute_refill, seconds_until_next_point
from app.modules.tap_mining.levels import get_level
from app.modules.tap_mining.task_verify import check_milestone, check_telegram_membership
from app.modules.tap_mining.upgrades import (
    UPGRADE_TYPES,
    effective_energy_cap,
    effective_points_per_tap,
    effective_regen_seconds,
    upgrade_cost,
)

router = APIRouter()


class TelegramAuthRequest(BaseModel):
    init_data: str
    referral_code: str | None = None


class TapRequest(BaseModel):
    tap_count: int = Field(gt=0)


class TapBeaconRequest(BaseModel):
    tap_count: int = Field(gt=0)
    access_token: str


class LinkWalletRequest(BaseModel):
    payout_address: str = Field(..., description="Bech32 SPRX address sprax1...")


class ClaimTokensRequest(BaseModel):
    points_to_claim: int | None = Field(default=None, gt=0, description="Optional points to claim, or max available if None")


async def _require_enabled(db: AsyncSession) -> None:
    """Feature gate — checks the admin-toggleable DB flag first (live, no
    redeploy needed), falling back to the static `.env` setting when the flag
    row hasn't been seeded yet (also what the test suite relies on, since it
    monkeypatches `settings.feature_tap_mining_enabled` without touching DB)."""
    result = await db.execute(select(FeatureFlag).where(FeatureFlag.name == "tap_mining_enabled"))
    flag = result.scalar_one_or_none()
    enabled = flag.is_enabled if flag is not None else settings.feature_tap_mining_enabled
    if not enabled:
        raise SpraxError(TAP_MINING_DISABLED, "Tap mining is not currently enabled", 403)


def _effective(miner: TapMiner) -> tuple[int, int, int]:
    """Returns (energy_cap, regen_seconds, points_per_tap) for this miner's upgrade levels."""
    return (
        effective_energy_cap(miner.energy_cap_level),
        effective_regen_seconds(miner.regen_speed_level),
        effective_points_per_tap(miner.multitap_level),
    )


def _serialize_profile(miner: TapMiner) -> dict:
    cap, regen_seconds, points_per_tap = _effective(miner)
    return {
        "telegram_id": miner.telegram_id,
        "username": miner.telegram_username,
        "first_name": miner.telegram_first_name,
        "last_name": miner.telegram_last_name,
        "photo_url": miner.telegram_photo_url,
        "points": miner.points,
        "energy": miner.energy,
        "energy_cap": cap,
        "energy_regen_seconds": regen_seconds,
        "points_per_tap": points_per_tap,
        "next_energy_in_seconds": seconds_until_next_point(miner.energy, miner.energy_last_refill_at, cap, regen_seconds),
        "taps_total": miner.taps_total,
        "checkin_streak": miner.checkin_streak,
        "longest_checkin_streak": miner.longest_checkin_streak,
        "checked_in_today": miner.last_checkin_at is not None and miner.last_checkin_at.date() == datetime.now(UTC).date(),
        "referral_code": miner.referral_code,
        "payout_address": miner.payout_address,
        "total_sprx_claimed": miner.total_sprx_claimed,
        "level": get_level(miner.points),
        "created_at": miner.created_at.isoformat(),
    }


async def _load_miner(miner_payload: dict, db: AsyncSession) -> TapMiner:
    miner_id = uuid.UUID(miner_payload["sub"])
    result = await db.execute(select(TapMiner).where(TapMiner.id == miner_id))
    miner = result.scalar_one_or_none()
    if miner is None:
        raise SpraxError(UNAUTHORIZED, "Tap miner profile not found", 401)
    return miner


def _apply_refill(miner: TapMiner, now: datetime) -> None:
    cap, regen_seconds, _ = _effective(miner)
    new_energy, new_refill_at = compute_refill(miner.energy, miner.energy_last_refill_at, cap, regen_seconds, now)
    miner.energy = new_energy
    miner.energy_last_refill_at = new_refill_at


def _log_activity(db: AsyncSession, miner_id: uuid.UUID, event_type: str, points_delta: int = 0, detail: str | None = None) -> None:
    db.add(TapMinerActivity(tap_miner_id=miner_id, event_type=event_type, points_delta=points_delta, detail=detail))


def _check_level_up(miner: TapMiner, points_before: int, db: AsyncSession) -> None:
    if get_level(miner.points)["index"] > get_level(points_before)["index"]:
        _log_activity(db, miner.id, "level_up", 0, get_level(miner.points)["name"])


async def _referred_count(miner: TapMiner, db: AsyncSession) -> int:
    result = await db.execute(select(func.count()).select_from(TapMiner).where(TapMiner.referred_by_id == miner.id))
    return result.scalar_one()


@router.post("/auth")
async def telegram_auth(req: TelegramAuthRequest, db: AsyncSession = Depends(get_db)):
    """Exchanges a Telegram WebApp `initData` string for a scoped access token,
    creating the tap-mining profile on first sign-in. Standalone identity —
    not linked to the wallet-based User model."""
    await _require_enabled(db)

    tg_user = validate_init_data(req.init_data)
    telegram_id = int(tg_user["id"])

    result = await db.execute(select(TapMiner).where(TapMiner.telegram_id == telegram_id))
    miner = result.scalar_one_or_none()

    now = datetime.now(UTC)
    if miner is None:
        miner = TapMiner(
            telegram_id=telegram_id,
            telegram_username=tg_user.get("username"),
            telegram_first_name=tg_user.get("first_name"),
            telegram_last_name=tg_user.get("last_name"),
            telegram_photo_url=tg_user.get("photo_url"),
            points=0,
            energy=settings.tap_mining_energy_cap,
            energy_last_refill_at=now,
            referral_code=uuid.uuid4().hex[:8].upper(),
        )
        db.add(miner)
        await db.flush()

        if req.referral_code:
            referrer_result = await db.execute(select(TapMiner).where(TapMiner.referral_code == req.referral_code))
            referrer = referrer_result.scalar_one_or_none()
            if referrer is not None and referrer.id != miner.id:
                referrer_points_before = referrer.points

                miner.referred_by_id = referrer.id
                miner.points += settings.tap_mining_referral_bonus_referee
                referrer.points += settings.tap_mining_referral_bonus_referrer
                referrer.referral_earnings += settings.tap_mining_referral_bonus_referrer

                who = f"@{tg_user.get('username')}" if tg_user.get("username") else f"Player {telegram_id}"
                _log_activity(db, referrer.id, "referral_joined", settings.tap_mining_referral_bonus_referrer, f"{who} joined")
                _log_activity(db, miner.id, "referral_bonus", settings.tap_mining_referral_bonus_referee, "Welcome bonus")
                _check_level_up(referrer, referrer_points_before, db)
                _check_level_up(miner, 0, db)
    else:
        miner.telegram_username = tg_user.get("username")
        miner.telegram_first_name = tg_user.get("first_name")
        miner.telegram_last_name = tg_user.get("last_name")
        miner.telegram_photo_url = tg_user.get("photo_url")
        _apply_refill(miner, now)

    if miner.is_banned:
        raise SpraxError(FORBIDDEN, "This account has been suspended", 403)

    await db.commit()

    access_token = create_access_token(str(miner.id), extra={"scope": "tap_mining", "telegram_id": telegram_id})
    return ApiResponse.ok({
        "access_token": access_token,
        "token_type": "bearer",
        "profile": _serialize_profile(miner),
    })


@router.get("/profile")
async def get_profile(miner_payload: dict = Depends(get_current_tap_miner), db: AsyncSession = Depends(get_db)):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    _apply_refill(miner, datetime.now(UTC))
    await db.commit()
    return ApiResponse.ok(_serialize_profile(miner))


async def _perform_tap(miner: TapMiner, tap_count: int, db: AsyncSession) -> dict:
    if miner.is_banned:
        raise SpraxError(FORBIDDEN, "This account has been suspended", 403)

    if tap_count > settings.tap_mining_max_taps_per_request:
        raise SpraxError(
            INVALID_REQUEST,
            f"tap_count exceeds max of {settings.tap_mining_max_taps_per_request} per request",
            400,
        )

    now = datetime.now(UTC)

    if miner.last_tap_at is not None:
        last_tap_at = miner.last_tap_at if miner.last_tap_at.tzinfo is not None else miner.last_tap_at.replace(tzinfo=UTC)
        elapsed_ms = (now - last_tap_at).total_seconds() * 1000
        if elapsed_ms < settings.tap_mining_min_tap_interval_ms:
            raise SpraxError(RATE_LIMITED, "Tapping too fast, slow down", 429)

    # Redis backs the per-user rate limit but isn't required for core gameplay —
    # if it's unreachable, skip the check rather than failing every tap. The
    # min-tap-interval and per-request cap above still bound abuse either way.
    try:
        rl_key = f"tap:rl:{miner.telegram_id}"
        rl_count = await redis_get(rl_key) or 0
        if rl_count >= settings.tap_mining_rate_limit_per_minute:
            raise SpraxError(RATE_LIMITED, "Too many tap requests, slow down", 429)
        await redis_set(rl_key, int(rl_count) + 1, ttl=60)
    except SpraxError:
        raise
    except Exception as e:
        logger.warning("tap_mining.rate_limit_unavailable", error=str(e))

    _apply_refill(miner, now)
    _, _, points_per_tap = _effective(miner)

    accepted = min(tap_count, miner.energy)
    points_earned = accepted * points_per_tap
    points_before = miner.points

    miner.energy -= accepted
    miner.points += points_earned
    miner.taps_total += accepted
    miner.last_tap_at = now

    _check_level_up(miner, points_before, db)

    await db.commit()

    cap, _, _ = _effective(miner)
    return {
        "points": miner.points,
        "energy": miner.energy,
        "energy_cap": cap,
        "points_earned": points_earned,
        "taps_accepted": accepted,
        "taps_rejected": tap_count - accepted,
    }


@router.post("/tap")
async def tap(
    req: TapRequest,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    result = await _perform_tap(miner, req.tap_count, db)
    return ApiResponse.ok(result)


@router.post("/tap/beacon")
async def tap_beacon(req: TapBeaconRequest, db: AsyncSession = Depends(get_db)):
    """Identical to POST /tap, but authenticates via a token in the JSON body
    instead of an Authorization header. `navigator.sendBeacon()` — the only
    reliable way to flush pending taps as the Mini App is closing, since
    Telegram's WebView doesn't consistently fire visibilitychange/beforeunload
    in time for a normal fetch to complete — cannot set custom headers, so the
    frontend's beacon flush hits this endpoint instead of the header-based one."""
    await _require_enabled(db)
    try:
        payload = verify_access_token(req.access_token)
    except Exception:
        raise SpraxError(UNAUTHORIZED, "Invalid or expired token", 401)
    if payload.get("scope") != "tap_mining":
        raise SpraxError(UNAUTHORIZED, "Invalid token scope", 401)

    miner = await _load_miner(payload, db)
    result = await _perform_tap(miner, req.tap_count, db)
    return ApiResponse.ok(result)


@router.post("/checkin")
async def checkin(miner_payload: dict = Depends(get_current_tap_miner), db: AsyncSession = Depends(get_db)):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)

    if miner.is_banned:
        raise SpraxError(FORBIDDEN, "This account has been suspended", 403)

    now = datetime.now(UTC)
    new_streak = next_streak(miner.checkin_streak, miner.last_checkin_at, now)
    if new_streak is None:
        raise SpraxError(ALREADY_CHECKED_IN, "Already checked in today", 409)

    reward = checkin_reward(new_streak)
    points_before = miner.points
    miner.points += reward
    miner.checkin_streak = new_streak
    miner.longest_checkin_streak = max(miner.longest_checkin_streak, new_streak)
    miner.last_checkin_at = now

    _log_activity(db, miner.id, "checkin", reward, f"Day {new_streak} streak")
    _check_level_up(miner, points_before, db)

    await db.commit()

    return ApiResponse.ok({
        "points": miner.points,
        "checkin_streak": miner.checkin_streak,
        "longest_checkin_streak": miner.longest_checkin_streak,
        "reward_earned": reward,
    })


@router.get("/leaderboard")
async def leaderboard(
    limit: int = 50,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    limit = max(1, min(limit, 100))

    result = await db.execute(select(TapMiner).order_by(TapMiner.points.desc()).limit(limit))
    top = result.scalars().all()

    rank_result = await db.execute(select(func.count()).select_from(TapMiner).where(TapMiner.points > miner.points))
    my_rank = rank_result.scalar_one() + 1

    return ApiResponse.ok({
        "leaderboard": [
            {
                "rank": i + 1,
                "telegram_id": m.telegram_id,
                "username": m.telegram_username,
                "points": m.points,
            }
            for i, m in enumerate(top)
        ],
        "me": {"rank": my_rank, "points": miner.points},
    })


@router.get("/referral")
async def get_referral(miner_payload: dict = Depends(get_current_tap_miner), db: AsyncSession = Depends(get_db)):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    referred_count = await _referred_count(miner, db)

    referral_link = f"{settings.tap_mining_webapp_url}?ref={miner.referral_code}" if settings.tap_mining_webapp_url else None

    return ApiResponse.ok({
        "referral_code": miner.referral_code,
        "referral_link": referral_link,
        "referred_count": referred_count,
        "referral_earnings": miner.referral_earnings,
    })


@router.get("/upgrades")
async def get_upgrades(miner_payload: dict = Depends(get_current_tap_miner), db: AsyncSession = Depends(get_db)):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)

    levels = {
        "energy_cap": miner.energy_cap_level,
        "regen_speed": miner.regen_speed_level,
        "multitap": miner.multitap_level,
    }
    effective_fns = {
        "energy_cap": effective_energy_cap,
        "regen_speed": effective_regen_seconds,
        "multitap": effective_points_per_tap,
    }

    out = []
    for upgrade_type in UPGRADE_TYPES:
        level = levels[upgrade_type]
        max_level = settings.tap_mining_upgrade_max_level
        out.append({
            "type": upgrade_type,
            "level": level,
            "max_level": max_level,
            "effective_value": effective_fns[upgrade_type](level),
            "next_cost": upgrade_cost(level) if level < max_level else None,
        })

    return ApiResponse.ok({"upgrades": out, "points": miner.points})


@router.post("/upgrades/{upgrade_type}")
async def purchase_upgrade(
    upgrade_type: str,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    await _require_enabled(db)
    if upgrade_type not in UPGRADE_TYPES:
        raise SpraxError(INVALID_REQUEST, f"Unknown upgrade type: {upgrade_type}", 400)

    miner = await _load_miner(miner_payload, db)
    level_attr = f"{upgrade_type}_level"
    current_level = getattr(miner, level_attr)

    if current_level >= settings.tap_mining_upgrade_max_level:
        raise SpraxError(INVALID_REQUEST, "Upgrade is already at max level", 400)

    cost = upgrade_cost(current_level)
    if miner.points < cost:
        raise SpraxError(INSUFFICIENT_POINTS, f"Requires {cost} points, you have {miner.points}", 400)

    miner.points -= cost
    setattr(miner, level_attr, current_level + 1)

    _log_activity(db, miner.id, "upgrade_purchased", -cost, f"{upgrade_type} → Lv.{current_level + 1}")

    await db.commit()

    return ApiResponse.ok({
        "type": upgrade_type,
        "level": current_level + 1,
        "points": miner.points,
    })


@router.get("/activity")
async def get_activity(
    limit: int = 20,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    limit = max(1, min(limit, 100))

    result = await db.execute(
        select(TapMinerActivity)
        .where(TapMinerActivity.tap_miner_id == miner.id)
        .order_by(TapMinerActivity.created_at.desc())
        .limit(limit)
    )
    rows = result.scalars().all()

    return ApiResponse.ok([
        {
            "event_type": row.event_type,
            "points_delta": row.points_delta,
            "detail": row.detail,
            "created_at": row.created_at.isoformat(),
        }
        for row in rows
    ])


@router.get("/tasks")
async def list_tasks(miner_payload: dict = Depends(get_current_tap_miner), db: AsyncSession = Depends(get_db)):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)

    tasks_result = await db.execute(
        select(TapMiningTask)
        .where(TapMiningTask.is_active == True)  # noqa: E712
        .order_by(TapMiningTask.sort_order, TapMiningTask.created_at)
    )
    tasks = tasks_result.scalars().all()

    completions_result = await db.execute(
        select(TapMinerTaskCompletion.task_id).where(TapMinerTaskCompletion.tap_miner_id == miner.id)
    )
    completed_ids = {row[0] for row in completions_result.all()}

    referred_count: int | None = None
    out = []
    for task in tasks:
        if task.id in completed_ids:
            status = "completed"
        elif task.task_type == "milestone":
            if referred_count is None:
                referred_count = await _referred_count(miner, db)
            status = "available" if check_milestone(task.verify_target, task.verify_value, miner, referred_count) else "locked"
        else:
            status = "available"

        out.append({
            "id": str(task.id),
            "title": task.title,
            "description": task.description,
            "task_type": task.task_type,
            "reward_points": task.reward_points,
            "action_url": task.action_url,
            "status": status,
        })

    return ApiResponse.ok(out)


@router.post("/tasks/{task_id}/claim")
async def claim_task(
    task_id: uuid.UUID,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)

    task_result = await db.execute(
        select(TapMiningTask).where(TapMiningTask.id == task_id, TapMiningTask.is_active == True)  # noqa: E712
    )
    task = task_result.scalar_one_or_none()
    if task is None:
        raise SpraxError(NOT_FOUND, "Task not found or inactive", 404)

    existing = await db.execute(
        select(TapMinerTaskCompletion).where(
            TapMinerTaskCompletion.tap_miner_id == miner.id,
            TapMinerTaskCompletion.task_id == task.id,
        )
    )
    if existing.scalar_one_or_none() is not None:
        raise SpraxError(TASK_ALREADY_CLAIMED, "Task already claimed", 409)

    if task.task_type == "telegram_join":
        verified = await check_telegram_membership(task.verify_target, miner.telegram_id)
        if not verified:
            raise SpraxError(INVALID_REQUEST, "Join the channel first, then claim", 400)
    elif task.task_type == "milestone":
        referred_count = await _referred_count(miner, db)
        if not check_milestone(task.verify_target, task.verify_value, miner, referred_count):
            raise SpraxError(INVALID_REQUEST, "You haven't reached this milestone yet", 400)

    points_before = miner.points
    miner.points += task.reward_points
    db.add(TapMinerTaskCompletion(tap_miner_id=miner.id, task_id=task.id))
    _log_activity(db, miner.id, "task_completed", task.reward_points, task.title)
    _check_level_up(miner, points_before, db)

    await db.commit()

    return ApiResponse.ok({"points": miner.points, "reward_earned": task.reward_points})


@router.post("/link-wallet")
async def link_wallet(
    req: LinkWalletRequest,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    """Link or update the player's on-chain payout address (Bech32 sprax1...)."""
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    if miner.is_banned:
        raise SpraxError(FORBIDDEN, "This account has been suspended", 403)

    norm_addr = validate_recipient_address(req.payout_address)
    miner.payout_address = norm_addr
    _log_activity(db, miner.id, "wallet_linked", 0, f"Linked to {norm_addr[:10]}...{norm_addr[-4:]}")
    await db.commit()

    return ApiResponse.ok({
        "payout_address": miner.payout_address,
        "message": "Wallet address linked successfully",
    })


@router.get("/claim-info")
async def get_claim_info(
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    """Query player's current claimable balance, conversion rate, and cooldown."""
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    now = datetime.now(UTC)

    cooldown_remaining = get_cooldown_remaining(miner.last_claimed_at, now)
    min_points = settings.tap_mining_claim_min_points
    rate = settings.tap_mining_points_per_sprx
    claimable_sprx, points_to_debit = calculate_payout(miner.points, rate)

    can_claim = (
        not miner.is_banned
        and miner.payout_address is not None
        and miner.points >= min_points
        and claimable_sprx > 0
        and cooldown_remaining == 0
    )

    return ApiResponse.ok({
        "payout_address": miner.payout_address,
        "points": miner.points,
        "min_points_required": min_points,
        "points_per_sprx": rate,
        "claimable_sprx": claimable_sprx,
        "points_to_debit": points_to_debit,
        "can_claim": can_claim,
        "cooldown_remaining_seconds": cooldown_remaining,
        "last_claimed_at": miner.last_claimed_at.isoformat() if miner.last_claimed_at else None,
        "total_sprx_claimed": miner.total_sprx_claimed,
    })


@router.post("/claim-tokens")
async def claim_tokens(
    req: ClaimTokensRequest | None = None,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    """Convert points to on-chain SPRX tokens and dispatches payout via Faucet."""
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)

    if miner.is_banned:
        raise SpraxError(FORBIDDEN, "This account has been suspended", 403)

    if not miner.payout_address:
        raise SpraxError(NO_PAYOUT_ADDRESS, "No payout wallet address linked. Call /link-wallet first.", 400)

    now = datetime.now(UTC)
    cooldown = get_cooldown_remaining(miner.last_claimed_at, now)
    if cooldown > 0:
        raise SpraxError(
            CLAIM_COOLDOWN_ACTIVE,
            f"Claim cooldown active. Please wait {cooldown} seconds.",
            429,
            detail={"retry_after_seconds": cooldown},
        )

    min_points = settings.tap_mining_claim_min_points
    points_to_convert = req.points_to_claim if (req and req.points_to_claim) else miner.points

    if points_to_convert > miner.points:
        raise SpraxError(
            INSUFFICIENT_POINTS,
            f"Requested {points_to_convert} points, but only {miner.points} available.",
            400,
        )

    if points_to_convert < min_points:
        raise SpraxError(
            INSUFFICIENT_POINTS,
            f"Minimum of {min_points} points required to claim tokens.",
            400,
        )

    rate = settings.tap_mining_points_per_sprx
    sprx_amount, points_debited = calculate_payout(points_to_convert, rate)

    if sprx_amount <= 0:
        raise SpraxError(
            INSUFFICIENT_POINTS,
            f"Insufficient points to claim at least 1 SPRX (rate: {rate} pts/SPRX).",
            400,
        )

    # Atomically debit points and record pending claim in DB
    miner.points -= points_debited
    miner.last_claimed_at = now
    miner.total_sprx_claimed += sprx_amount

    claim_record = TapMinerClaim(
        tap_miner_id=miner.id,
        points_spent=points_debited,
        sprx_amount=sprx_amount,
        recipient_address=miner.payout_address,
        status="pending",
    )
    db.add(claim_record)
    await db.flush()

    # Dispatch to Faucet service
    try:
        payout_res = await dispatch_faucet_payout(
            recipient=miner.payout_address,
            amount=sprx_amount,
        )
        claim_record.status = "completed"
        claim_record.tx_hash = payout_res.get("tx_hash") or payout_res.get("transaction_hash")
        claim_record.completed_at = datetime.now(UTC)
        _log_activity(
            db,
            miner.id,
            "tokens_claimed",
            -points_debited,
            f"Claimed {sprx_amount} tSPRX (Tx: {claim_record.tx_hash or 'confirmed'})",
        )
        await db.commit()

        return ApiResponse.ok({
            "claim_id": str(claim_record.id),
            "status": "completed",
            "sprx_amount": sprx_amount,
            "points_debited": points_debited,
            "remaining_points": miner.points,
            "tx_hash": claim_record.tx_hash,
            "recipient_address": miner.payout_address,
        })
    except Exception as e:
        # If faucet dispatch fails, mark claim failed and refund points so player does not lose balance
        claim_record.status = "failed"
        claim_record.error_message = str(e)
        miner.points += points_debited
        miner.total_sprx_claimed -= sprx_amount
        miner.last_claimed_at = None
        await db.commit()
        if isinstance(e, SpraxError):
            raise e
        raise SpraxError(FAUCET_DISPATCH_FAILED, f"Token payout failed: {e}", 502)


@router.get("/claims")
async def get_claims_history(
    limit: int = 20,
    miner_payload: dict = Depends(get_current_tap_miner),
    db: AsyncSession = Depends(get_db),
):
    """List historical on-chain token claims for the current player."""
    await _require_enabled(db)
    miner = await _load_miner(miner_payload, db)
    limit = max(1, min(limit, 100))

    result = await db.execute(
        select(TapMinerClaim)
        .where(TapMinerClaim.tap_miner_id == miner.id)
        .order_by(TapMinerClaim.created_at.desc())
        .limit(limit)
    )
    claims = result.scalars().all()

    return ApiResponse.ok([
        {
            "id": str(c.id),
            "points_spent": c.points_spent,
            "sprx_amount": c.sprx_amount,
            "recipient_address": c.recipient_address,
            "status": c.status,
            "tx_hash": c.tx_hash,
            "error_message": c.error_message,
            "created_at": c.created_at.isoformat(),
            "completed_at": c.completed_at.isoformat() if c.completed_at else None,
        }
        for c in claims
    ])

