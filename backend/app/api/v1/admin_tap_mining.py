import uuid
from datetime import UTC, datetime, timedelta

import httpx
from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy import String, cast, func, or_, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import settings
from app.core.dependencies import get_db, require_admin
from app.core.response import ApiResponse
from app.db.models.admin import AuditLog
from app.db.models.tap_mining import TapMiner, TapMinerActivity, TapMinerClaim, TapMinerTaskCompletion, TapMiningTask
from app.modules.tap_mining.claim import dispatch_faucet_payout
from app.modules.tap_mining.levels import get_level
from app.modules.tap_mining.task_verify import MILESTONE_KINDS
from app.modules.tap_mining.upgrades import effective_energy_cap

TASK_TYPES = ("manual", "telegram_join", "milestone")

router = APIRouter()


def _serialize_player(miner: TapMiner) -> dict:
    return {
        "telegram_id": miner.telegram_id,
        "username": miner.telegram_username,
        "points": miner.points,
        "energy": miner.energy,
        "level": get_level(miner.points)["name"],
        "is_banned": miner.is_banned,
        "taps_total": miner.taps_total,
        "checkin_streak": miner.checkin_streak,
        "referral_code": miner.referral_code,
        "payout_address": miner.payout_address,
        "total_sprx_claimed": miner.total_sprx_claimed,
        "created_at": miner.created_at.isoformat(),
        "last_tap_at": miner.last_tap_at.isoformat() if miner.last_tap_at else None,
    }


async def _get_miner_or_404(telegram_id: int, db: AsyncSession) -> TapMiner:
    result = await db.execute(select(TapMiner).where(TapMiner.telegram_id == telegram_id))
    miner = result.scalar_one_or_none()
    if miner is None:
        raise HTTPException(404, f"No tap-mining player with telegram_id {telegram_id}")
    return miner


@router.get("/dashboard")
async def get_tap_mining_dashboard(user: dict = Depends(require_admin), db: AsyncSession = Depends(get_db)):
    total_players = (await db.execute(select(func.count()).select_from(TapMiner))).scalar_one()
    total_points_issued = (await db.execute(select(func.coalesce(func.sum(TapMiner.points), 0)))).scalar_one()
    total_taps = (await db.execute(select(func.coalesce(func.sum(TapMiner.taps_total), 0)))).scalar_one()
    banned_count = (
        await db.execute(select(func.count()).select_from(TapMiner).where(TapMiner.is_banned == True))  # noqa: E712
    ).scalar_one()
    dau = (
        await db.execute(
            select(func.count())
            .select_from(TapMiner)
            .where(TapMiner.last_tap_at >= datetime.now(UTC) - timedelta(hours=24))
        )
    ).scalar_one()

    total_sprx_claimed = (
        await db.execute(
            select(func.coalesce(func.sum(TapMinerClaim.sprx_amount), 0))
            .where(TapMinerClaim.status == "completed")
        )
    ).scalar_one()

    total_claims_count = (
        await db.execute(
            select(func.count())
            .select_from(TapMinerClaim)
            .where(TapMinerClaim.status == "completed")
        )
    ).scalar_one()

    pending_claims_count = (
        await db.execute(
            select(func.count())
            .select_from(TapMinerClaim)
            .where(TapMinerClaim.status == "pending")
        )
    ).scalar_one()

    top_result = await db.execute(select(TapMiner).order_by(TapMiner.points.desc()).limit(5))
    top_earners = [
        {"telegram_id": m.telegram_id, "username": m.telegram_username, "points": m.points}
        for m in top_result.scalars().all()
    ]

    return ApiResponse.ok({
        "total_players": total_players,
        "total_points_issued": total_points_issued,
        "total_taps": total_taps,
        "banned_count": banned_count,
        "dau": dau,
        "total_sprx_claimed": total_sprx_claimed,
        "total_claims_count": total_claims_count,
        "pending_claims_count": pending_claims_count,
        "top_earners": top_earners,
    })


@router.get("/players")
async def list_tap_mining_players(
    search: str | None = Query(None),
    limit: int = Query(50, le=200),
    offset: int = Query(0),
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    query = select(TapMiner)
    if search:
        query = query.where(
            or_(
                TapMiner.telegram_username.ilike(f"%{search}%"),
                cast(TapMiner.telegram_id, String).like(f"%{search}%"),
            )
        )

    total = (await db.execute(select(func.count()).select_from(query.subquery()))).scalar_one()
    result = await db.execute(query.order_by(TapMiner.points.desc()).offset(offset).limit(limit))
    players = result.scalars().all()

    return ApiResponse.ok({"total": total, "players": [_serialize_player(m) for m in players]})


@router.get("/players/{telegram_id}")
async def get_tap_mining_player(
    telegram_id: int,
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    miner = await _get_miner_or_404(telegram_id, db)

    activity_result = await db.execute(
        select(TapMinerActivity)
        .where(TapMinerActivity.tap_miner_id == miner.id)
        .order_by(TapMinerActivity.created_at.desc())
        .limit(50)
    )
    activity = [
        {
            "event_type": row.event_type,
            "points_delta": row.points_delta,
            "detail": row.detail,
            "created_at": row.created_at.isoformat(),
        }
        for row in activity_result.scalars().all()
    ]

    return ApiResponse.ok({
        **_serialize_player(miner),
        "energy_cap_level": miner.energy_cap_level,
        "regen_speed_level": miner.regen_speed_level,
        "multitap_level": miner.multitap_level,
        "referral_earnings": miner.referral_earnings,
        "referred_by_id": str(miner.referred_by_id) if miner.referred_by_id else None,
        "activity": activity,
    })


async def _set_banned(telegram_id: int, banned: bool, user: dict, db: AsyncSession) -> dict:
    miner = await _get_miner_or_404(telegram_id, db)
    miner.is_banned = banned

    db.add(AuditLog(
        actor=user.get("username", "admin"),
        action="tap_mining_ban" if banned else "tap_mining_unban",
        resource=str(telegram_id),
        result="success",
    ))
    await db.commit()
    return {"telegram_id": telegram_id, "is_banned": miner.is_banned}


@router.post("/players/{telegram_id}/ban")
async def ban_tap_mining_player(
    telegram_id: int,
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    return ApiResponse.ok(await _set_banned(telegram_id, True, user, db))


@router.post("/players/{telegram_id}/unban")
async def unban_tap_mining_player(
    telegram_id: int,
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    return ApiResponse.ok(await _set_banned(telegram_id, False, user, db))


class AdjustPlayerRequest(BaseModel):
    points_delta: int = 0
    energy_delta: int = 0
    reason: str


@router.post("/players/{telegram_id}/adjust")
async def adjust_tap_mining_player(
    telegram_id: int,
    req: AdjustPlayerRequest,
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    if not req.reason.strip():
        raise HTTPException(400, "A reason is required for manual adjustments")

    miner = await _get_miner_or_404(telegram_id, db)
    points_before, energy_before = miner.points, miner.energy

    energy_cap = effective_energy_cap(miner.energy_cap_level)
    miner.points = max(0, miner.points + req.points_delta)
    miner.energy = max(0, min(energy_cap, miner.energy + req.energy_delta))

    applied_points_delta = miner.points - points_before
    applied_energy_delta = miner.energy - energy_before

    if applied_points_delta != 0:
        db.add(TapMinerActivity(
            tap_miner_id=miner.id,
            event_type="admin_adjustment",
            points_delta=applied_points_delta,
            detail=req.reason,
        ))

    db.add(AuditLog(
        actor=user.get("username", "admin"),
        action="tap_mining_adjust",
        resource=str(telegram_id),
        result="success",
        extra_data={
            "reason": req.reason,
            "points_before": points_before,
            "points_after": miner.points,
            "energy_before": energy_before,
            "energy_after": miner.energy,
            "requested_points_delta": req.points_delta,
            "requested_energy_delta": req.energy_delta,
        },
    ))

    await db.commit()

    return ApiResponse.ok({
        "telegram_id": telegram_id,
        "points": miner.points,
        "energy": miner.energy,
        "applied_points_delta": applied_points_delta,
        "applied_energy_delta": applied_energy_delta,
    })


def _serialize_task(task: TapMiningTask, completions: int) -> dict:
    return {
        "id": str(task.id),
        "title": task.title,
        "description": task.description,
        "task_type": task.task_type,
        "reward_points": task.reward_points,
        "action_url": task.action_url,
        "verify_target": task.verify_target,
        "verify_value": task.verify_value,
        "is_active": task.is_active,
        "sort_order": task.sort_order,
        "completions": completions,
        "created_at": task.created_at.isoformat(),
    }


def _validate_task_fields(task_type: str, verify_target: str | None, verify_value: int | None) -> None:
    if task_type not in TASK_TYPES:
        raise HTTPException(400, f"Invalid task_type: {task_type}")
    if task_type == "milestone":
        if verify_target not in MILESTONE_KINDS or verify_value is None:
            raise HTTPException(400, f"milestone tasks need verify_target in {MILESTONE_KINDS} and a verify_value")
    if task_type == "telegram_join" and not verify_target:
        raise HTTPException(400, "telegram_join tasks need verify_target (the chat id/username)")


@router.get("/tasks")
async def list_tap_mining_tasks(user: dict = Depends(require_admin), db: AsyncSession = Depends(get_db)):
    tasks_result = await db.execute(select(TapMiningTask).order_by(TapMiningTask.sort_order, TapMiningTask.created_at))
    tasks = tasks_result.scalars().all()

    counts_result = await db.execute(
        select(TapMinerTaskCompletion.task_id, func.count()).group_by(TapMinerTaskCompletion.task_id)
    )
    counts = dict(counts_result.all())

    return ApiResponse.ok([_serialize_task(t, counts.get(t.id, 0)) for t in tasks])


class CreateTaskRequest(BaseModel):
    title: str = Field(max_length=200)
    description: str | None = Field(default=None, max_length=5000)
    task_type: str
    reward_points: int = 0
    action_url: str | None = Field(default=None, max_length=500)
    verify_target: str | None = Field(default=None, max_length=200)
    verify_value: int | None = None
    is_active: bool = True
    sort_order: int = 0


@router.post("/tasks")
async def create_tap_mining_task(
    req: CreateTaskRequest,
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    _validate_task_fields(req.task_type, req.verify_target, req.verify_value)

    task = TapMiningTask(**req.model_dump())
    db.add(task)
    await db.flush()

    db.add(AuditLog(
        actor=user.get("username", "admin"),
        action="tap_mining_task_create",
        resource=str(task.id),
        result="success",
        extra_data=req.model_dump(),
    ))
    await db.commit()
    return ApiResponse.ok(_serialize_task(task, 0))


class UpdateTaskRequest(BaseModel):
    title: str | None = Field(default=None, max_length=200)
    description: str | None = Field(default=None, max_length=5000)
    reward_points: int | None = None
    action_url: str | None = Field(default=None, max_length=500)
    verify_target: str | None = Field(default=None, max_length=200)
    verify_value: int | None = None
    is_active: bool | None = None
    sort_order: int | None = None



@router.patch("/tasks/{task_id}")
async def update_tap_mining_task(
    task_id: uuid.UUID,
    req: UpdateTaskRequest,
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    result = await db.execute(select(TapMiningTask).where(TapMiningTask.id == task_id))
    task = result.scalar_one_or_none()
    if task is None:
        raise HTTPException(404, "Task not found")

    updates = req.model_dump(exclude_unset=True)
    _validate_task_fields(
        task.task_type,
        updates.get("verify_target", task.verify_target),
        updates.get("verify_value", task.verify_value),
    )
    for key, value in updates.items():
        setattr(task, key, value)

    db.add(AuditLog(
        actor=user.get("username", "admin"),
        action="tap_mining_task_update",
        resource=str(task_id),
        result="success",
        extra_data=updates,
    ))
    await db.commit()

    count_result = await db.execute(
        select(func.count()).select_from(TapMinerTaskCompletion).where(TapMinerTaskCompletion.task_id == task.id)
    )
    return ApiResponse.ok(_serialize_task(task, count_result.scalar_one()))


@router.get("/faucet/status")
async def get_faucet_status(user: dict = Depends(require_admin)):
    """Fetch live health, balance, and operational statistics from the SPRX Faucet daemon."""
    faucet_url = settings.sprax_faucet_url.rstrip("/")
    try:
        async with httpx.AsyncClient(timeout=4.0) as client:
            resp = await client.get(f"{faucet_url}/stats")
            if resp.status_code == 200:
                stats = resp.json()
                return ApiResponse.ok({
                    "online": True,
                    **stats,
                    "points_per_sprx": settings.tap_mining_points_per_sprx,
                    "min_claim_points": settings.tap_mining_claim_min_points,
                    "claim_cooldown_seconds": settings.tap_mining_claim_cooldown_seconds,
                })
    except Exception:
        pass

    return ApiResponse.ok({
        "online": False,
        "faucet_url": faucet_url,
        "points_per_sprx": settings.tap_mining_points_per_sprx,
        "min_claim_points": settings.tap_mining_claim_min_points,
        "claim_cooldown_seconds": settings.tap_mining_claim_cooldown_seconds,
    })


@router.get("/claims")
async def list_tap_mining_claims(
    status: str | None = Query(None),
    search: str | None = Query(None),
    limit: int = Query(50, le=200),
    offset: int = Query(0),
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    """List and filter on-chain player claims across all statuses."""
    query = (
        select(TapMinerClaim, TapMiner.telegram_id, TapMiner.telegram_username)
        .join(TapMiner, TapMiner.id == TapMinerClaim.tap_miner_id)
    )

    if status:
        query = query.where(TapMinerClaim.status == status)

    if search:
        search_pattern = f"%{search}%"
        query = query.where(
            or_(
                TapMinerClaim.recipient_address.ilike(search_pattern),
                TapMinerClaim.tx_hash.ilike(search_pattern),
                TapMiner.telegram_username.ilike(search_pattern),
                cast(TapMiner.telegram_id, String).like(search_pattern),
            )
        )

    total = (await db.execute(select(func.count()).select_from(query.subquery()))).scalar_one()
    result = await db.execute(query.order_by(TapMinerClaim.created_at.desc()).offset(offset).limit(limit))
    rows = result.all()

    claims = []
    for claim, tg_id, tg_username in rows:
        claims.append({
            "id": str(claim.id),
            "tap_miner_id": str(claim.tap_miner_id),
            "telegram_id": tg_id,
            "telegram_username": tg_username,
            "points_spent": claim.points_spent,
            "sprx_amount": claim.sprx_amount,
            "recipient_address": claim.recipient_address,
            "status": claim.status,
            "tx_hash": claim.tx_hash,
            "error_message": claim.error_message,
            "created_at": claim.created_at.isoformat(),
            "completed_at": claim.completed_at.isoformat() if claim.completed_at else None,
        })

    return ApiResponse.ok({"total": total, "claims": claims})


@router.post("/claims/{claim_id}/retry")
async def retry_failed_claim(
    claim_id: uuid.UUID,
    user: dict = Depends(require_admin),
    db: AsyncSession = Depends(get_db),
):
    """Admin tool to retry an on-chain payout for a claim that previously failed."""
    result = await db.execute(
        select(TapMinerClaim, TapMiner)
        .join(TapMiner, TapMiner.id == TapMinerClaim.tap_miner_id)
        .where(TapMinerClaim.id == claim_id)
    )
    row = result.first()
    if not row:
        raise HTTPException(404, "Claim not found")

    claim, miner = row
    if claim.status == "completed":
        raise HTTPException(400, "Claim has already completed")

    if miner.points < claim.points_spent:
        raise HTTPException(400, f"Player points ({miner.points}) are insufficient for {claim.points_spent} pts claim")

    miner.points -= claim.points_spent
    miner.total_sprx_claimed += claim.sprx_amount
    claim.status = "pending"
    claim.error_message = None
    await db.flush()

    try:
        payout_res = await dispatch_faucet_payout(claim.recipient_address, claim.sprx_amount)
        claim.status = "completed"
        claim.tx_hash = payout_res.get("tx_hash") or payout_res.get("transaction_hash")
        claim.completed_at = datetime.now(UTC)
        db.add(AuditLog(
            actor=user.get("username", "admin"),
            action="tap_mining_claim_retry",
            resource=str(claim.id),
            result="success",
            extra_data={"tx_hash": claim.tx_hash, "sprx_amount": claim.sprx_amount},
        ))
        await db.commit()
        return ApiResponse.ok({"status": "completed", "tx_hash": claim.tx_hash})
    except Exception as e:
        claim.status = "failed"
        claim.error_message = str(e)
        miner.points += claim.points_spent
        miner.total_sprx_claimed -= claim.sprx_amount
        await db.commit()
        raise HTTPException(502, f"Faucet payout retry failed: {e}")

