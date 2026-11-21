import hashlib
import hmac
import json
import time
import uuid
from urllib.parse import urlencode

import pytest
from httpx import AsyncClient
from sqlalchemy import select

import app.api.v1.tap_mining as tap_mining_module
from app.core.config import settings
from app.core.security import create_access_token

BOT_TOKEN = "test-bot-token"


@pytest.fixture(autouse=True)
def fake_redis(monkeypatch):
    """No Redis service in this test environment — stub the rate-limit
    counter helpers used by /tap (see test_tap_mining.py for the same)."""
    store: dict[str, object] = {}

    async def fake_get(key: str):
        return store.get(key)

    async def fake_set(key: str, value, ttl=None):
        store[key] = value

    monkeypatch.setattr(tap_mining_module, "redis_get", fake_get)
    monkeypatch.setattr(tap_mining_module, "redis_set", fake_set)


def build_init_data(telegram_id: int, username: str = "tapper") -> str:
    user = {"id": telegram_id, "username": username, "first_name": "Tap"}
    fields = {
        "user": json.dumps(user, separators=(",", ":")),
        "auth_date": str(int(time.time())),
        "query_id": uuid.uuid4().hex,
    }
    data_check_string = "\n".join(f"{k}={v}" for k, v in sorted(fields.items()))
    secret_key = hmac.new(b"WebAppData", BOT_TOKEN.encode(), hashlib.sha256).digest()
    fields["hash"] = hmac.new(secret_key, data_check_string.encode(), hashlib.sha256).hexdigest()
    return urlencode(fields)


@pytest.fixture(autouse=True)
def tap_mining_settings(monkeypatch):
    monkeypatch.setattr(settings, "feature_tap_mining_enabled", True)
    monkeypatch.setattr(settings, "telegram_bot_token", BOT_TOKEN)
    monkeypatch.setattr(settings, "tap_mining_energy_cap", 1000)


def admin_headers() -> dict:
    token = create_access_token("admin-id", extra={"role": "admin", "username": "admin_user"})
    return {"Authorization": f"Bearer {token}"}


def viewer_headers() -> dict:
    token = create_access_token("viewer-id", extra={"role": "viewer", "username": "viewer_user"})
    return {"Authorization": f"Bearer {token}"}


async def _create_player(client: AsyncClient, telegram_id: int, points: int = 0) -> None:
    resp = await client.post("/api/v1/tap-mining/auth", json={"init_data": build_init_data(telegram_id)})
    assert resp.status_code == 200
    if points:
        token = resp.json()["data"]["access_token"]
        # Use the player's own energy_cap (1000, from fixture) to earn points via taps.
        await client.post(
            "/api/v1/tap-mining/tap",
            json={"tap_count": min(points, 1000)},
            headers={"Authorization": f"Bearer {token}"},
        )


@pytest.mark.asyncio
async def test_admin_endpoints_require_auth(client: AsyncClient):
    resp = await client.get("/api/v1/admin/tap-mining/dashboard")
    assert resp.status_code == 401


@pytest.mark.asyncio
async def test_viewer_forbidden_on_ban(client: AsyncClient):
    await _create_player(client, 700001)
    resp = await client.post("/api/v1/admin/tap-mining/players/700001/ban", headers=viewer_headers())
    assert resp.status_code == 403


@pytest.mark.asyncio
async def test_dashboard_totals(client: AsyncClient):
    await _create_player(client, 700002, points=50)
    await _create_player(client, 700003, points=20)
    # Ensure player 700002 is in the top earners list across all test runs
    await client.post(
        "/api/v1/admin/tap-mining/players/700002/adjust",
        json={"points_delta": 100_000, "reason": "test top earner"},
        headers=admin_headers(),
    )

    resp = await client.get("/api/v1/admin/tap-mining/dashboard", headers=admin_headers())
    assert resp.status_code == 200
    data = resp.json()["data"]
    assert data["total_players"] >= 2
    assert data["total_points_issued"] >= 100_000
    assert any(row["telegram_id"] == 700002 for row in data["top_earners"])


@pytest.mark.asyncio
async def test_players_search_by_username_and_id(client: AsyncClient):
    await _create_player(client, 700004)

    by_id = await client.get("/api/v1/admin/tap-mining/players?search=700004", headers=admin_headers())
    assert any(p["telegram_id"] == 700004 for p in by_id.json()["data"]["players"])

    by_username = await client.get("/api/v1/admin/tap-mining/players?search=tapper", headers=admin_headers())
    assert any(p["telegram_id"] == 700004 for p in by_username.json()["data"]["players"])


@pytest.mark.asyncio
async def test_player_detail_includes_activity(client: AsyncClient):
    await _create_player(client, 700005)
    token_resp = await client.post("/api/v1/tap-mining/auth", json={"init_data": build_init_data(700005)})
    token = token_resp.json()["data"]["access_token"]
    await client.post("/api/v1/tap-mining/checkin", headers={"Authorization": f"Bearer {token}"})

    resp = await client.get("/api/v1/admin/tap-mining/players/700005", headers=admin_headers())
    assert resp.status_code == 200
    data = resp.json()["data"]
    assert data["telegram_id"] == 700005
    assert any(row["event_type"] == "checkin" for row in data["activity"])


@pytest.mark.asyncio
async def test_player_not_found(client: AsyncClient):
    resp = await client.get("/api/v1/admin/tap-mining/players/999999999", headers=admin_headers())
    assert resp.status_code == 404


@pytest.mark.asyncio
async def test_ban_blocks_subsequent_auth_and_unban_restores(client: AsyncClient):
    await _create_player(client, 700006)

    ban_resp = await client.post("/api/v1/admin/tap-mining/players/700006/ban", headers=admin_headers())
    assert ban_resp.status_code == 200
    assert ban_resp.json()["data"]["is_banned"] is True

    auth_after_ban = await client.post("/api/v1/tap-mining/auth", json={"init_data": build_init_data(700006)})
    assert auth_after_ban.status_code == 403
    assert auth_after_ban.json()["error"]["code"] == "FORBIDDEN"

    unban_resp = await client.post("/api/v1/admin/tap-mining/players/700006/unban", headers=admin_headers())
    assert unban_resp.status_code == 200
    assert unban_resp.json()["data"]["is_banned"] is False

    auth_after_unban = await client.post("/api/v1/tap-mining/auth", json={"init_data": build_init_data(700006)})
    assert auth_after_unban.status_code == 200


@pytest.mark.asyncio
async def test_adjust_applies_delta_and_requires_reason(client: AsyncClient):
    await _create_player(client, 700007, points=10)

    no_reason = await client.post(
        "/api/v1/admin/tap-mining/players/700007/adjust",
        json={"points_delta": 5, "energy_delta": 0, "reason": ""},
        headers=admin_headers(),
    )
    assert no_reason.status_code == 400

    resp = await client.post(
        "/api/v1/admin/tap-mining/players/700007/adjust",
        json={"points_delta": 100, "energy_delta": -5, "reason": "compensation for bug"},
        headers=admin_headers(),
    )
    assert resp.status_code == 200
    data = resp.json()["data"]
    assert data["applied_points_delta"] == 100
    assert data["applied_energy_delta"] == -5

    detail = await client.get("/api/v1/admin/tap-mining/players/700007", headers=admin_headers())
    activity = detail.json()["data"]["activity"]
    assert any(row["event_type"] == "admin_adjustment" and row["detail"] == "compensation for bug" for row in activity)


@pytest.mark.asyncio
async def test_adjust_clamps_points_at_zero(client: AsyncClient):
    await _create_player(client, 700008, points=10)

    resp = await client.post(
        "/api/v1/admin/tap-mining/players/700008/adjust",
        json={"points_delta": -999999, "energy_delta": 0, "reason": "test clamp"},
        headers=admin_headers(),
    )
    assert resp.status_code == 200
    assert resp.json()["data"]["points"] == 0


# --- Tasks ---

@pytest.mark.asyncio
async def test_create_task_requires_admin(client: AsyncClient):
    resp = await client.post(
        "/api/v1/admin/tap-mining/tasks",
        json={"title": "Follow us", "task_type": "manual", "reward_points": 25},
        headers=viewer_headers(),
    )
    assert resp.status_code == 403


@pytest.mark.asyncio
async def test_create_task_validates_type(client: AsyncClient):
    resp = await client.post(
        "/api/v1/admin/tap-mining/tasks",
        json={"title": "Bad", "task_type": "not-a-real-type", "reward_points": 25},
        headers=admin_headers(),
    )
    assert resp.status_code == 400


@pytest.mark.asyncio
async def test_create_task_validates_milestone_fields(client: AsyncClient):
    resp = await client.post(
        "/api/v1/admin/tap-mining/tasks",
        json={"title": "Bad milestone", "task_type": "milestone", "reward_points": 25},
        headers=admin_headers(),
    )
    assert resp.status_code == 400


@pytest.mark.asyncio
async def test_create_update_and_list_task_with_completion_count(client: AsyncClient):
    create_resp = await client.post(
        "/api/v1/admin/tap-mining/tasks",
        json={"title": "Follow us", "task_type": "manual", "reward_points": 25},
        headers=admin_headers(),
    )
    assert create_resp.status_code == 200
    task_id = create_resp.json()["data"]["id"]

    await _create_player(client, 700009)
    token_resp = await client.post("/api/v1/tap-mining/auth", json={"init_data": build_init_data(700009)})
    player_token = token_resp.json()["data"]["access_token"]
    await client.post(
        f"/api/v1/tap-mining/tasks/{task_id}/claim", headers={"Authorization": f"Bearer {player_token}"}
    )

    listing = (await client.get("/api/v1/admin/tap-mining/tasks", headers=admin_headers())).json()["data"]
    listed = next(t for t in listing if t["id"] == task_id)
    assert listed["completions"] == 1

    update_resp = await client.patch(
        f"/api/v1/admin/tap-mining/tasks/{task_id}",
        json={"is_active": False, "reward_points": 50},
        headers=admin_headers(),
    )
    assert update_resp.status_code == 200
    assert update_resp.json()["data"]["is_active"] is False
    assert update_resp.json()["data"]["reward_points"] == 50


@pytest.mark.asyncio
async def test_admin_dashboard_and_claims_listing(client: AsyncClient, db_session, monkeypatch):
    import app.api.v1.admin_tap_mining as admin_tap_mining_module
    from app.db.models.tap_mining import TapMiner, TapMinerClaim

    # Create a player with a claim
    await _create_player(client, 800001)
    miner = (await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == 800001))).scalar_one()

    claim = TapMinerClaim(
        tap_miner_id=miner.id,
        points_spent=2000,
        sprx_amount=2,
        recipient_address="sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq2pfe2q",
        status="completed",
        tx_hash="0x" + "11" * 32,
    )
    db_session.add(claim)
    await db_session.commit()

    # Check dashboard includes claims stats
    dash_resp = await client.get("/api/v1/admin/tap-mining/dashboard", headers=admin_headers())
    assert dash_resp.status_code == 200
    dash_data = dash_resp.json()["data"]
    assert dash_data["total_sprx_claimed"] >= 2
    assert dash_data["total_claims_count"] >= 1

    # Check claims list
    claims_resp = await client.get("/api/v1/admin/tap-mining/claims", headers=admin_headers())
    assert claims_resp.status_code == 200
    claims_data = claims_resp.json()["data"]
    assert claims_data["total"] >= 1
    c = next(x for x in claims_data["claims"] if x["id"] == str(claim.id))
    assert c["sprx_amount"] == 2
    assert c["telegram_id"] == 800001
    assert c["status"] == "completed"

    # Check faucet status endpoint
    faucet_resp = await client.get("/api/v1/admin/tap-mining/faucet/status", headers=admin_headers())
    assert faucet_resp.status_code == 200
    assert "points_per_sprx" in faucet_resp.json()["data"]

