import hashlib
import hmac
import json
import time
import uuid
from datetime import UTC, datetime, timedelta
from urllib.parse import urlencode

import pytest
from httpx import AsyncClient
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

import app.api.v1.tap_mining as tap_mining_module
from app.core.config import settings
from app.db.models.tap_mining import TapMiner, TapMiningTask

BOT_TOKEN = "test-bot-token"


@pytest.fixture(autouse=True)
def fake_redis(monkeypatch):
    """CI has no Redis service for the Python test job — stub the two
    helpers the tap endpoint uses for its per-user rate-limit counter."""
    store: dict[str, object] = {}

    async def fake_get(key: str):
        return store.get(key)

    async def fake_set(key: str, value, ttl=None):
        store[key] = value

    monkeypatch.setattr(tap_mining_module, "redis_get", fake_get)
    monkeypatch.setattr(tap_mining_module, "redis_set", fake_set)


def build_init_data(telegram_id: int, username: str = "tapper", auth_date: int | None = None) -> str:
    user = {"id": telegram_id, "username": username, "first_name": "Tap"}
    fields = {
        "user": json.dumps(user, separators=(",", ":")),
        "auth_date": str(auth_date if auth_date is not None else int(time.time())),
        "query_id": uuid.uuid4().hex,
    }
    data_check_string = "\n".join(f"{k}={v}" for k, v in sorted(fields.items()))
    secret_key = hmac.new(b"WebAppData", BOT_TOKEN.encode(), hashlib.sha256).digest()
    computed_hash = hmac.new(secret_key, data_check_string.encode(), hashlib.sha256).hexdigest()
    fields["hash"] = computed_hash
    return urlencode(fields)


@pytest.fixture(autouse=True)
def tap_mining_settings(monkeypatch):
    monkeypatch.setattr(settings, "feature_tap_mining_enabled", True)
    monkeypatch.setattr(settings, "telegram_bot_token", BOT_TOKEN)
    monkeypatch.setattr(settings, "tap_mining_energy_cap", 10)
    monkeypatch.setattr(settings, "tap_mining_energy_regen_seconds", 3)
    monkeypatch.setattr(settings, "tap_mining_max_taps_per_request", 5)
    monkeypatch.setattr(settings, "tap_mining_min_tap_interval_ms", 0)
    monkeypatch.setattr(settings, "tap_mining_rate_limit_per_minute", 1000)
    monkeypatch.setattr(settings, "tap_mining_checkin_base_points", 10)
    monkeypatch.setattr(settings, "tap_mining_checkin_streak_bonus", 5)
    monkeypatch.setattr(settings, "tap_mining_referral_bonus_referrer", 100)
    monkeypatch.setattr(settings, "tap_mining_referral_bonus_referee", 50)
    monkeypatch.setattr(settings, "tap_mining_upgrade_max_level", 3)
    monkeypatch.setattr(settings, "tap_mining_upgrade_base_cost", 5)
    monkeypatch.setattr(settings, "tap_mining_upgrade_cost_multiplier", 2.0)
    monkeypatch.setattr(settings, "tap_mining_upgrade_energy_cap_step", 5)
    monkeypatch.setattr(settings, "tap_mining_upgrade_regen_step", 1)
    monkeypatch.setattr(settings, "tap_mining_upgrade_min_regen_seconds", 1)
    monkeypatch.setattr(settings, "tap_mining_upgrade_multitap_step", 1)


async def _auth(client: AsyncClient, telegram_id: int, referral_code: str | None = None) -> dict:
    payload = {"init_data": build_init_data(telegram_id)}
    if referral_code:
        payload["referral_code"] = referral_code
    resp = await client.post("/api/v1/tap-mining/auth", json=payload)
    assert resp.status_code == 200, resp.text
    return resp.json()["data"]


async def _auth_token(client: AsyncClient, telegram_id: int) -> str:
    return (await _auth(client, telegram_id))["access_token"]


async def _get_miner(db_session: AsyncSession, telegram_id: int) -> TapMiner:
    result = await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == telegram_id))
    return result.scalar_one()


async def _create_task(db_session: AsyncSession, **kwargs) -> TapMiningTask:
    defaults = {"title": "Test Task", "task_type": "manual", "reward_points": 25}
    task = TapMiningTask(**{**defaults, **kwargs})
    db_session.add(task)
    await db_session.commit()
    await db_session.refresh(task)
    return task


@pytest.mark.asyncio
async def test_auth_creates_profile_with_full_energy(client: AsyncClient):
    resp = await client.post("/api/v1/tap-mining/auth", json={"init_data": build_init_data(111)})
    assert resp.status_code == 200
    profile = resp.json()["data"]["profile"]
    assert profile["telegram_id"] == 111
    assert profile["energy"] == 10
    assert profile["points"] == 0


@pytest.mark.asyncio
async def test_auth_rejects_bad_signature(client: AsyncClient):
    bad_init_data = build_init_data(112) + "x"
    resp = await client.post("/api/v1/tap-mining/auth", json={"init_data": bad_init_data})
    assert resp.status_code == 401
    assert resp.json()["error"]["code"] == "INVALID_TELEGRAM_AUTH"


@pytest.mark.asyncio
async def test_tap_earns_points_and_spends_energy(client: AsyncClient):
    token = await _auth_token(client, 113)
    headers = {"Authorization": f"Bearer {token}"}

    resp = await client.post("/api/v1/tap-mining/tap", json={"tap_count": 4}, headers=headers)
    assert resp.status_code == 200
    data = resp.json()["data"]
    assert data["taps_accepted"] == 4
    assert data["points"] == 4
    assert data["energy"] == 6


@pytest.mark.asyncio
async def test_tap_is_capped_by_available_energy(client: AsyncClient, monkeypatch):
    monkeypatch.setattr(settings, "tap_mining_max_taps_per_request", 50)
    token = await _auth_token(client, 114)
    headers = {"Authorization": f"Bearer {token}"}

    resp = await client.post("/api/v1/tap-mining/tap", json={"tap_count": 50}, headers=headers)
    assert resp.status_code == 200
    data = resp.json()["data"]
    assert data["taps_accepted"] == 10
    assert data["taps_rejected"] == 40
    assert data["energy"] == 0


@pytest.mark.asyncio
async def test_tap_rejects_over_per_request_cap(client: AsyncClient):
    token = await _auth_token(client, 115)
    headers = {"Authorization": f"Bearer {token}"}

    resp = await client.post("/api/v1/tap-mining/tap", json={"tap_count": 6}, headers=headers)
    assert resp.status_code == 400
    assert resp.json()["error"]["code"] == "INVALID_REQUEST"


@pytest.mark.asyncio
async def test_wallet_token_cannot_access_tap_mining(client: AsyncClient):
    resp = await client.get("/api/v1/tap-mining/profile", headers={"Authorization": "Bearer not-a-real-token"})
    assert resp.status_code == 401


@pytest.mark.asyncio
async def test_unauthorized_access(client: AsyncClient):
    resp = await client.get("/api/v1/tap-mining/profile")
    assert resp.status_code == 401


# --- Daily check-in ---

@pytest.mark.asyncio
async def test_checkin_first_time_awards_base_reward(client: AsyncClient):
    token = await _auth_token(client, 201)
    headers = {"Authorization": f"Bearer {token}"}

    resp = await client.post("/api/v1/tap-mining/checkin", headers=headers)
    assert resp.status_code == 200, resp.text
    data = resp.json()["data"]
    assert data["checkin_streak"] == 1
    assert data["reward_earned"] == 10
    assert data["points"] == 10


@pytest.mark.asyncio
async def test_checkin_twice_same_day_rejected(client: AsyncClient):
    token = await _auth_token(client, 202)
    headers = {"Authorization": f"Bearer {token}"}

    await client.post("/api/v1/tap-mining/checkin", headers=headers)
    resp = await client.post("/api/v1/tap-mining/checkin", headers=headers)
    assert resp.status_code == 409
    assert resp.json()["error"]["code"] == "ALREADY_CHECKED_IN"


@pytest.mark.asyncio
async def test_checkin_streak_increments_on_consecutive_day(client: AsyncClient, db_session: AsyncSession):
    token = await _auth_token(client, 203)
    headers = {"Authorization": f"Bearer {token}"}

    await client.post("/api/v1/tap-mining/checkin", headers=headers)

    miner = await _get_miner(db_session, 203)
    miner.last_checkin_at = datetime.now(UTC) - timedelta(days=1)
    await db_session.commit()

    resp = await client.post("/api/v1/tap-mining/checkin", headers=headers)
    assert resp.status_code == 200
    data = resp.json()["data"]
    assert data["checkin_streak"] == 2
    assert data["reward_earned"] == 15  # base 10 + bonus 5 * (2-1)


@pytest.mark.asyncio
async def test_checkin_streak_resets_after_gap(client: AsyncClient, db_session: AsyncSession):
    token = await _auth_token(client, 204)
    headers = {"Authorization": f"Bearer {token}"}

    await client.post("/api/v1/tap-mining/checkin", headers=headers)

    miner = await _get_miner(db_session, 204)
    miner.last_checkin_at = datetime.now(UTC) - timedelta(days=3)
    await db_session.commit()

    resp = await client.post("/api/v1/tap-mining/checkin", headers=headers)
    assert resp.status_code == 200
    assert resp.json()["data"]["checkin_streak"] == 1


# --- Referrals ---

@pytest.mark.asyncio
async def test_referral_bonus_awarded_on_first_login(client: AsyncClient):
    referrer_data = await _auth(client, 301)
    referrer_code = referrer_data["profile"]["referral_code"]

    referee_data = await _auth(client, 302, referral_code=referrer_code)
    assert referee_data["profile"]["points"] == 50

    referrer_headers = {"Authorization": f"Bearer {referrer_data['access_token']}"}
    profile_resp = await client.get("/api/v1/tap-mining/profile", headers=referrer_headers)
    assert profile_resp.json()["data"]["points"] == 100

    referral_resp = await client.get("/api/v1/tap-mining/referral", headers=referrer_headers)
    referral_data = referral_resp.json()["data"]
    assert referral_data["referred_count"] == 1
    assert referral_data["referral_earnings"] == 100


@pytest.mark.asyncio
async def test_referral_unknown_code_ignored(client: AsyncClient):
    data = await _auth(client, 303, referral_code="NOTAREALCODE")
    assert data["profile"]["points"] == 0


@pytest.mark.asyncio
async def test_referral_bonus_only_applied_on_first_login(client: AsyncClient):
    referrer_data = await _auth(client, 304)
    referrer_code = referrer_data["profile"]["referral_code"]

    referee_data = await _auth(client, 305, referral_code=referrer_code)
    assert referee_data["profile"]["points"] == 50

    # Second login for the same telegram_id must not re-award the bonus.
    referee_again = await _auth(client, 305, referral_code=referrer_code)
    assert referee_again["profile"]["points"] == 50


# --- Leaderboard ---

@pytest.mark.asyncio
async def test_leaderboard_orders_by_points_and_reports_rank(client: AsyncClient):
    # The test DB is shared (session-scoped) across the whole test run, so other
    # tests' miners may already outrank these two — assert relative ordering
    # between A and B rather than an absolute top-of-leaderboard position.
    token_a = await _auth_token(client, 401)
    token_b = await _auth_token(client, 402)

    await client.post(
        "/api/v1/tap-mining/tap", json={"tap_count": 5}, headers={"Authorization": f"Bearer {token_a}"}
    )
    await client.post(
        "/api/v1/tap-mining/tap", json={"tap_count": 2}, headers={"Authorization": f"Bearer {token_b}"}
    )

    resp = await client.get(
        "/api/v1/tap-mining/leaderboard?limit=100", headers={"Authorization": f"Bearer {token_a}"}
    )
    assert resp.status_code == 200
    data = resp.json()["data"]
    board = data["leaderboard"]

    rank_a = next(row["rank"] for row in board if row["telegram_id"] == 401)
    rank_b = next(row["rank"] for row in board if row["telegram_id"] == 402)
    assert rank_a < rank_b
    assert data["me"]["rank"] == rank_a
    assert data["me"]["points"] == 5


# --- Upgrades ---

@pytest.mark.asyncio
async def test_upgrade_purchase_deducts_points_and_raises_effective_cap(client: AsyncClient, db_session: AsyncSession):
    token = await _auth_token(client, 501)
    headers = {"Authorization": f"Bearer {token}"}

    miner = await _get_miner(db_session, 501)
    miner.points = 100
    await db_session.commit()

    resp = await client.post("/api/v1/tap-mining/upgrades/energy_cap", headers=headers)
    assert resp.status_code == 200, resp.text
    data = resp.json()["data"]
    assert data["level"] == 1
    assert data["points"] == 95  # base_cost=5 at level 0

    profile_resp = await client.get("/api/v1/tap-mining/profile", headers=headers)
    assert profile_resp.json()["data"]["energy_cap"] == 15  # base 10 + step 5


@pytest.mark.asyncio
async def test_upgrade_insufficient_points_rejected(client: AsyncClient):
    token = await _auth_token(client, 502)
    headers = {"Authorization": f"Bearer {token}"}

    resp = await client.post("/api/v1/tap-mining/upgrades/multitap", headers=headers)
    assert resp.status_code == 400
    assert resp.json()["error"]["code"] == "INSUFFICIENT_POINTS"


@pytest.mark.asyncio
async def test_upgrade_unknown_type_rejected(client: AsyncClient):
    token = await _auth_token(client, 503)
    headers = {"Authorization": f"Bearer {token}"}

    resp = await client.post("/api/v1/tap-mining/upgrades/not-a-real-upgrade", headers=headers)
    assert resp.status_code == 400
    assert resp.json()["error"]["code"] == "INVALID_REQUEST"


@pytest.mark.asyncio
async def test_upgrade_max_level_rejected(client: AsyncClient, db_session: AsyncSession):
    token = await _auth_token(client, 504)
    headers = {"Authorization": f"Bearer {token}"}

    miner = await _get_miner(db_session, 504)
    miner.points = 100000
    miner.regen_speed_level = settings.tap_mining_upgrade_max_level
    await db_session.commit()

    resp = await client.post("/api/v1/tap-mining/upgrades/regen_speed", headers=headers)
    assert resp.status_code == 400
    assert resp.json()["error"]["code"] == "INVALID_REQUEST"


# --- Profile: level, created_at, activity log ---

@pytest.mark.asyncio
async def test_profile_includes_level_and_join_date(client: AsyncClient):
    resp = await _auth(client, 601)
    profile = resp["profile"]
    assert profile["level"]["name"] == "Bronze"
    assert profile["level"]["index"] == 0
    assert profile["created_at"] is not None


@pytest.mark.asyncio
async def test_checkin_logs_activity(client: AsyncClient):
    token = await _auth_token(client, 602)
    headers = {"Authorization": f"Bearer {token}"}

    await client.post("/api/v1/tap-mining/checkin", headers=headers)

    resp = await client.get("/api/v1/tap-mining/activity", headers=headers)
    assert resp.status_code == 200
    rows = resp.json()["data"]
    assert len(rows) == 1
    assert rows[0]["event_type"] == "checkin"
    assert rows[0]["points_delta"] == 10
    assert rows[0]["detail"] == "Day 1 streak"


@pytest.mark.asyncio
async def test_referral_logs_activity_for_both_sides(client: AsyncClient):
    referrer_data = await _auth(client, 603)
    referrer_code = referrer_data["profile"]["referral_code"]
    referrer_headers = {"Authorization": f"Bearer {referrer_data['access_token']}"}

    referee_data = await _auth(client, 604, referral_code=referrer_code)
    referee_headers = {"Authorization": f"Bearer {referee_data['access_token']}"}

    referrer_activity = (await client.get("/api/v1/tap-mining/activity", headers=referrer_headers)).json()["data"]
    assert any(row["event_type"] == "referral_joined" and row["points_delta"] == 100 for row in referrer_activity)

    referee_activity = (await client.get("/api/v1/tap-mining/activity", headers=referee_headers)).json()["data"]
    assert any(row["event_type"] == "referral_bonus" and row["points_delta"] == 50 for row in referee_activity)


@pytest.mark.asyncio
async def test_upgrade_purchase_logs_activity(client: AsyncClient, db_session: AsyncSession):
    token = await _auth_token(client, 605)
    headers = {"Authorization": f"Bearer {token}"}

    miner = await _get_miner(db_session, 605)
    miner.points = 100
    await db_session.commit()

    await client.post("/api/v1/tap-mining/upgrades/multitap", headers=headers)

    resp = await client.get("/api/v1/tap-mining/activity", headers=headers)
    rows = resp.json()["data"]
    assert len(rows) == 1
    assert rows[0]["event_type"] == "upgrade_purchased"
    assert rows[0]["points_delta"] == -5
    assert rows[0]["detail"] == "multitap → Lv.1"


@pytest.mark.asyncio
async def test_level_up_logged_once_when_crossing_threshold(client: AsyncClient, db_session: AsyncSession):
    token = await _auth_token(client, 606)
    headers = {"Authorization": f"Bearer {token}"}

    miner = await _get_miner(db_session, 606)
    miner.points = 995  # just below the Silver threshold (1000)
    await db_session.commit()

    resp = await client.post("/api/v1/tap-mining/tap", json={"tap_count": 5}, headers=headers)
    assert resp.status_code == 200
    assert resp.json()["data"]["points"] == 1000

    activity = (await client.get("/api/v1/tap-mining/activity", headers=headers)).json()["data"]
    level_ups = [row for row in activity if row["event_type"] == "level_up"]
    assert len(level_ups) == 1
    assert level_ups[0]["detail"] == "Silver"


@pytest.mark.asyncio
async def test_no_level_up_logged_when_threshold_not_crossed(client: AsyncClient, db_session: AsyncSession):
    token = await _auth_token(client, 607)
    headers = {"Authorization": f"Bearer {token}"}

    miner = await _get_miner(db_session, 607)
    miner.points = 100
    await db_session.commit()

    await client.post("/api/v1/tap-mining/tap", json={"tap_count": 3}, headers=headers)

    activity = (await client.get("/api/v1/tap-mining/activity", headers=headers)).json()["data"]
    assert not any(row["event_type"] == "level_up" for row in activity)


@pytest.mark.asyncio
async def test_activity_respects_limit(client: AsyncClient, db_session: AsyncSession):
    # SQLite's CURRENT_TIMESTAMP (used for created_at server_default in tests)
    # only has 1-second resolution, so rows inserted within the same test can
    # tie on created_at — assert count/limit behavior, not exact tie-break order
    # (Postgres, used in production, has microsecond resolution and orders correctly).
    token = await _auth_token(client, 608)
    headers = {"Authorization": f"Bearer {token}"}

    miner = await _get_miner(db_session, 608)
    miner.points = 100000
    await db_session.commit()

    expected_details = set()
    for upgrade_type in ("energy_cap", "regen_speed", "multitap"):
        await client.post(f"/api/v1/tap-mining/upgrades/{upgrade_type}", headers=headers)
        expected_details.add(f"{upgrade_type} → Lv.1")

    full = (await client.get("/api/v1/tap-mining/activity?limit=100", headers=headers)).json()["data"]
    assert {row["detail"] for row in full} == expected_details

    limited = (await client.get("/api/v1/tap-mining/activity?limit=2", headers=headers)).json()["data"]
    assert len(limited) == 2
    assert {row["detail"] for row in limited}.issubset(expected_details)


# --- Tasks ---

@pytest.mark.asyncio
async def test_manual_task_claim_once_then_conflict(client: AsyncClient, db_session: AsyncSession):
    task = await _create_task(db_session, title="Follow us", task_type="manual", reward_points=25)
    token = await _auth_token(client, 609)
    headers = {"Authorization": f"Bearer {token}"}

    resp = await client.post(f"/api/v1/tap-mining/tasks/{task.id}/claim", headers=headers)
    assert resp.status_code == 200, resp.text
    assert resp.json()["data"]["reward_earned"] == 25

    again = await client.post(f"/api/v1/tap-mining/tasks/{task.id}/claim", headers=headers)
    assert again.status_code == 409
    assert again.json()["error"]["code"] == "TASK_ALREADY_CLAIMED"


@pytest.mark.asyncio
async def test_telegram_join_task_requires_verification(client: AsyncClient, db_session: AsyncSession, monkeypatch):
    task = await _create_task(
        db_session, title="Join channel", task_type="telegram_join", reward_points=40, verify_target="@sprxcoin_channel"
    )
    token = await _auth_token(client, 610)
    headers = {"Authorization": f"Bearer {token}"}

    async def not_joined(chat_id, telegram_user_id):
        return False

    monkeypatch.setattr(tap_mining_module, "check_telegram_membership", not_joined)
    denied = await client.post(f"/api/v1/tap-mining/tasks/{task.id}/claim", headers=headers)
    assert denied.status_code == 400

    async def joined(chat_id, telegram_user_id):
        return True

    monkeypatch.setattr(tap_mining_module, "check_telegram_membership", joined)
    resp = await client.post(f"/api/v1/tap-mining/tasks/{task.id}/claim", headers=headers)
    assert resp.status_code == 200
    assert resp.json()["data"]["reward_earned"] == 40


@pytest.mark.asyncio
async def test_milestone_task_gated_on_checkin_streak(client: AsyncClient, db_session: AsyncSession):
    task = await _create_task(
        db_session, title="3-day streak", task_type="milestone", reward_points=60,
        verify_target="checkin_streak", verify_value=3,
    )
    token = await _auth_token(client, 611)
    headers = {"Authorization": f"Bearer {token}"}

    not_yet = await client.post(f"/api/v1/tap-mining/tasks/{task.id}/claim", headers=headers)
    assert not_yet.status_code == 400

    miner = await _get_miner(db_session, 611)
    miner.checkin_streak = 3
    await db_session.commit()

    resp = await client.post(f"/api/v1/tap-mining/tasks/{task.id}/claim", headers=headers)
    assert resp.status_code == 200
    assert resp.json()["data"]["reward_earned"] == 60


@pytest.mark.asyncio
async def test_tasks_list_reports_status_per_type(client: AsyncClient, db_session: AsyncSession):
    manual_task = await _create_task(db_session, title="Manual", task_type="manual", reward_points=10)
    milestone_task = await _create_task(
        db_session, title="Reach streak 5", task_type="milestone", reward_points=10,
        verify_target="checkin_streak", verify_value=5,
    )
    token = await _auth_token(client, 612)
    headers = {"Authorization": f"Bearer {token}"}

    listing = (await client.get("/api/v1/tap-mining/tasks", headers=headers)).json()["data"]
    by_id = {row["id"]: row for row in listing}
    assert by_id[str(manual_task.id)]["status"] == "available"
    assert by_id[str(milestone_task.id)]["status"] == "locked"

    await client.post(f"/api/v1/tap-mining/tasks/{manual_task.id}/claim", headers=headers)
    listing_after = (await client.get("/api/v1/tap-mining/tasks", headers=headers)).json()["data"]
    by_id_after = {row["id"]: row for row in listing_after}
    assert by_id_after[str(manual_task.id)]["status"] == "completed"


@pytest.mark.asyncio
async def test_claim_nonexistent_task_404(client: AsyncClient):
    token = await _auth_token(client, 613)
    headers = {"Authorization": f"Bearer {token}"}
    resp = await client.post(
        "/api/v1/tap-mining/tasks/00000000-0000-0000-0000-000000000000/claim", headers=headers
    )
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_claim_inactive_task_404(client: AsyncClient, db_session: AsyncSession):
    task = await _create_task(db_session, title="Retired task", task_type="manual", is_active=False)
    token = await _auth_token(client, 614)
    headers = {"Authorization": f"Bearer {token}"}
    resp = await client.post(f"/api/v1/tap-mining/tasks/{task.id}/claim", headers=headers)
    assert resp.status_code == 404
