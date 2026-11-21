import hashlib
import hmac
import json
import time
import uuid
from datetime import UTC, datetime, timedelta
from urllib.parse import urlencode

import pytest
from httpx import AsyncClient, Response
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

import app.api.v1.tap_mining as tap_mining_module
from app.core.config import settings
from app.db.models.tap_mining import TapMiner, TapMinerActivity, TapMinerClaim

BOT_TOKEN = "test-bot-token"
VALID_SPRAX_ADDR = "sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq2pfe2q"  # 44-char valid Bech32 address
VALID_SPRAX_ADDR_2 = "sprax1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqm79wve"


@pytest.fixture(autouse=True)
def fake_redis(monkeypatch):
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
    computed_hash = hmac.new(secret_key, data_check_string.encode(), hashlib.sha256).hexdigest()
    fields["hash"] = computed_hash
    return urlencode(fields)


@pytest.fixture(autouse=True)
def tap_claim_settings(monkeypatch):
    monkeypatch.setattr(settings, "feature_tap_mining_enabled", True)
    monkeypatch.setattr(settings, "telegram_bot_token", BOT_TOKEN)
    monkeypatch.setattr(settings, "tap_mining_claim_min_points", 1000)
    monkeypatch.setattr(settings, "tap_mining_points_per_sprx", 1000)
    monkeypatch.setattr(settings, "tap_mining_claim_cooldown_seconds", 86400)
    monkeypatch.setattr(settings, "sprax_faucet_url", "http://faucet.test:8000")


async def get_auth_token(client: AsyncClient, telegram_id: int) -> str:
    res = await client.post("/api/v1/tap-mining/auth", json={"init_data": build_init_data(telegram_id)})
    assert res.status_code == 200
    return res.json()["data"]["access_token"]


@pytest.mark.asyncio
async def test_link_wallet_success_and_invalid_formats(client: AsyncClient, db_session: AsyncSession):
    token = await get_auth_token(client, 10001)
    headers = {"Authorization": f"Bearer {token}"}

    # 1. Reject invalid address formats
    bad_res = await client.post(
        "/api/v1/tap-mining/link-wallet",
        json={"payout_address": "0x1234567890abcdef"},
        headers=headers,
    )
    assert bad_res.status_code == 400
    assert bad_res.json()["error"]["code"] == "INVALID_ADDRESS"

    short_res = await client.post(
        "/api/v1/tap-mining/link-wallet",
        json={"payout_address": "sprax1short"},
        headers=headers,
    )
    assert short_res.status_code == 400
    assert short_res.json()["error"]["code"] == "INVALID_ADDRESS"

    # 2. Accept valid address
    ok_res = await client.post(
        "/api/v1/tap-mining/link-wallet",
        json={"payout_address": VALID_SPRAX_ADDR},
        headers=headers,
    )
    assert ok_res.status_code == 200
    assert ok_res.json()["data"]["payout_address"] == VALID_SPRAX_ADDR

    # 3. Verify reflected in profile and activity
    prof_res = await client.get("/api/v1/tap-mining/profile", headers=headers)
    assert prof_res.json()["data"]["payout_address"] == VALID_SPRAX_ADDR


@pytest.mark.asyncio
async def test_claim_info_states(client: AsyncClient, db_session: AsyncSession):
    tg_id = 10002
    token = await get_auth_token(client, tg_id)
    headers = {"Authorization": f"Bearer {token}"}

    # Unlinked wallet -> can_claim is False
    res1 = await client.get("/api/v1/tap-mining/claim-info", headers=headers)
    data1 = res1.json()["data"]
    assert data1["can_claim"] is False
    assert data1["payout_address"] is None
    assert data1["claimable_sprx"] == 0

    # Link wallet but points = 0 -> can_claim is False
    await client.post(
        "/api/v1/tap-mining/link-wallet",
        json={"payout_address": VALID_SPRAX_ADDR},
        headers=headers,
    )
    res2 = await client.get("/api/v1/tap-mining/claim-info", headers=headers)
    data2 = res2.json()["data"]
    assert data2["can_claim"] is False
    assert data2["payout_address"] == VALID_SPRAX_ADDR

    # Give miner 3,500 points
    miner_result = await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == tg_id))
    miner = miner_result.scalar_one()
    miner.points = 3500
    await db_session.commit()

    res3 = await client.get("/api/v1/tap-mining/claim-info", headers=headers)
    data3 = res3.json()["data"]
    assert data3["can_claim"] is True
    assert data3["points"] == 3500
    assert data3["claimable_sprx"] == 3
    assert data3["points_to_debit"] == 3000
    assert data3["cooldown_remaining_seconds"] == 0


@pytest.mark.asyncio
async def test_claim_tokens_validation_checks(client: AsyncClient, db_session: AsyncSession):
    tg_id = 10003
    token = await get_auth_token(client, tg_id)
    headers = {"Authorization": f"Bearer {token}"}

    # 1. No payout address -> 400 NO_PAYOUT_ADDRESS
    err_no_wallet = await client.post("/api/v1/tap-mining/claim-tokens", headers=headers)
    assert err_no_wallet.status_code == 400
    assert err_no_wallet.json()["error"]["code"] == "NO_PAYOUT_ADDRESS"

    # Link wallet
    await client.post(
        "/api/v1/tap-mining/link-wallet",
        json={"payout_address": VALID_SPRAX_ADDR},
        headers=headers,
    )

    # 2. Points below minimum (0 < 1000) -> 400 INSUFFICIENT_POINTS
    err_low_points = await client.post("/api/v1/tap-mining/claim-tokens", headers=headers)
    assert err_low_points.status_code == 400
    assert err_low_points.json()["error"]["code"] == "INSUFFICIENT_POINTS"

    # 3. Active cooldown check
    miner_res = await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == tg_id))
    miner = miner_res.scalar_one()
    miner.points = 5000
    miner.last_claimed_at = datetime.now(UTC) - timedelta(hours=2)  # 22 hours remaining
    await db_session.commit()

    err_cooldown = await client.post("/api/v1/tap-mining/claim-tokens", headers=headers)
    assert err_cooldown.status_code == 429
    assert err_cooldown.json()["error"]["code"] == "CLAIM_COOLDOWN_ACTIVE"


@pytest.mark.asyncio
async def test_claim_tokens_successful_flow(client: AsyncClient, db_session: AsyncSession, monkeypatch):
    tg_id = 10004
    token = await get_auth_token(client, tg_id)
    headers = {"Authorization": f"Bearer {token}"}

    # Setup player with 2,500 points and linked wallet
    await client.post(
        "/api/v1/tap-mining/link-wallet",
        json={"payout_address": VALID_SPRAX_ADDR},
        headers=headers,
    )
    miner_res = await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == tg_id))
    miner = miner_res.scalar_one()
    miner.points = 2500
    await db_session.commit()

    # Mock Faucet Dispatch response
    fake_tx_hash = "0x" + "aa" * 32

    async def mock_faucet_payout(recipient, amount, faucet_url=None, client=None):
        assert recipient == VALID_SPRAX_ADDR
        assert amount == 2  # 2,500 // 1,000 = 2 SPRX
        return {
            "success": True,
            "recipient": recipient,
            "amount": amount,
            "tx_hash": fake_tx_hash,
        }

    monkeypatch.setattr(tap_mining_module, "dispatch_faucet_payout", mock_faucet_payout)

    # Execute claim
    claim_res = await client.post("/api/v1/tap-mining/claim-tokens", headers=headers)
    assert claim_res.status_code == 200
    claim_data = claim_res.json()["data"]

    assert claim_data["status"] == "completed"
    assert claim_data["sprx_amount"] == 2
    assert claim_data["points_debited"] == 2000
    assert claim_data["remaining_points"] == 500
    assert claim_data["tx_hash"] == fake_tx_hash

    # Verify miner state updated in DB
    miner_db = (await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == tg_id))).scalar_one()
    assert miner_db.points == 500
    assert miner_db.total_sprx_claimed == 2
    assert miner_db.last_claimed_at is not None

    # Verify claim record in database
    claim_rows = (await db_session.execute(select(TapMinerClaim).where(TapMinerClaim.tap_miner_id == miner_db.id))).scalars().all()
    assert len(claim_rows) == 1
    assert claim_rows[0].status == "completed"
    assert claim_rows[0].points_spent == 2000
    assert claim_rows[0].sprx_amount == 2
    assert claim_rows[0].tx_hash == fake_tx_hash

    # Verify activity logged
    act_rows = (await db_session.execute(select(TapMinerActivity).where(TapMinerActivity.tap_miner_id == miner_db.id))).scalars().all()
    assert any(a.event_type == "tokens_claimed" and a.points_delta == -2000 for a in act_rows)

    # Verify /claims history endpoint
    hist_res = await client.get("/api/v1/tap-mining/claims", headers=headers)
    assert hist_res.status_code == 200
    hist = hist_res.json()["data"]
    assert len(hist) == 1
    assert hist[0]["sprx_amount"] == 2
    assert hist[0]["tx_hash"] == fake_tx_hash


@pytest.mark.asyncio
async def test_claim_tokens_faucet_failure_refunds_points(client: AsyncClient, db_session: AsyncSession, monkeypatch):
    tg_id = 10005
    token = await get_auth_token(client, tg_id)
    headers = {"Authorization": f"Bearer {token}"}

    # Setup player with 3,000 points and linked wallet
    await client.post(
        "/api/v1/tap-mining/link-wallet",
        json={"payout_address": VALID_SPRAX_ADDR},
        headers=headers,
    )
    miner_res = await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == tg_id))
    miner = miner_res.scalar_one()
    miner.points = 3000
    await db_session.commit()

    # Mock Faucet failure
    async def mock_failing_faucet(recipient, amount, faucet_url=None, client=None):
        raise tap_mining_module.SpraxError(
            tap_mining_module.FAUCET_DISPATCH_FAILED,
            "Faucet test simulated outage",
            502,
        )

    monkeypatch.setattr(tap_mining_module, "dispatch_faucet_payout", mock_failing_faucet)

    # Execute claim -> should return 502 and refund points
    claim_res = await client.post("/api/v1/tap-mining/claim-tokens", headers=headers)
    assert claim_res.status_code == 502
    assert claim_res.json()["error"]["code"] == "FAUCET_DISPATCH_FAILED"

    # Verify miner points were NOT lost
    miner_db = (await db_session.execute(select(TapMiner).where(TapMiner.telegram_id == tg_id))).scalar_one()
    assert miner_db.points == 3000
    assert miner_db.total_sprx_claimed == 0
    assert miner_db.last_claimed_at is None

    # Verify claim record marked as failed
    claim_rows = (await db_session.execute(select(TapMinerClaim).where(TapMinerClaim.tap_miner_id == miner_db.id))).scalars().all()
    assert len(claim_rows) == 1
    assert claim_rows[0].status == "failed"
    assert "Faucet test simulated outage" in (claim_rows[0].error_message or "")

