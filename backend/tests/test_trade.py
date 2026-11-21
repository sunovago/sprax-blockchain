from datetime import UTC, datetime, timedelta
from types import SimpleNamespace
from unittest.mock import AsyncMock

import pytest
from pydantic import ValidationError

from app.api.v1 import trade
from app.core.exceptions import SpraxError


@pytest.fixture
def settings(monkeypatch):
    settings = SimpleNamespace(
        trade_gateway_url="https://gateway.example",
        trade_gateway_token="test-token",
        trade_checkout_hosts="checkout.example",
        secret_key="test-only-secret-at-least-32-characters",
    )
    monkeypatch.setattr(trade, "get_settings", lambda: settings)
    return settings


def option(mode="fiat", side="buy"):
    return {
        "id": "sprx-inr",
        "label": "SPRX with INR / UPI",
        "mode": mode,
        "side": side,
        "asset": "SPRX",
        "payment_asset": "INR" if mode == "fiat" else "USDT",
        "payment_method": "UPI" if mode == "fiat" else "Wallet",
        "chain_id": "sprax-devnet-1",
        "min_amount": "10",
        "max_amount": "100000",
    }


def request():
    return trade.QuoteRequest(
        option_id="sprx-inr",
        amount="100.123456789123456789",
        wallet_address="sprax1testaddress",
        chain_id="sprax-devnet-1",
    )


def quote(side="buy", payment="INR"):
    return {
        "quote_id": "provider-quote-1",
        "provider": "Test provider",
        "pay_amount": request().amount,
        "pay_asset": payment if side == "buy" else "SPRX",
        "receive_amount": "5",
        "receive_asset": "SPRX" if side == "buy" else payment,
        "fee_amount": "1",
        "fee_asset": payment,
        "expires_at": (datetime.now(UTC) + timedelta(minutes=2)).isoformat(),
    }


@pytest.mark.asyncio
async def test_disabled_provider_does_not_offer_trades(settings):
    settings.trade_gateway_token = ""
    result = await trade.capabilities()
    assert result.data["options"] == []


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "mode,side", [("fiat", "buy"), ("fiat", "sell"), ("spot", "buy"), ("spot", "sell")]
)
async def test_quote_review_and_idempotent_hosted_checkout(settings, monkeypatch, mode, side):
    gateway = AsyncMock(
        side_effect=[
            {"provider": "Test provider", "options": [option(mode, side)]},
            quote(side, "INR" if mode == "fiat" else "USDT"),
            {"checkout_url": "https://checkout.example/session/1"},
            {"checkout_url": "https://checkout.example/session/1"},
        ]
    )
    monkeypatch.setattr(trade, "gateway", gateway)
    result = await trade.quote(request())
    assert result.data["pay_amount"] == request().amount
    session_request = trade.SessionRequest(quote_token=result.data["quote_token"])
    assert (await trade.session(session_request)).data["status"] == "awaiting_provider_confirmation"
    await trade.session(session_request)
    assert gateway.call_args_list[2].args[2] == gateway.call_args_list[3].args[2]


@pytest.mark.asyncio
async def test_wrong_network_and_pair_are_rejected(settings, monkeypatch):
    monkeypatch.setattr(
        trade, "gateway", AsyncMock(return_value={"provider": "Test", "options": [option()]})
    )
    with pytest.raises(SpraxError, match="pair or payment method"):
        await trade.quote(request().model_copy(update={"chain_id": "other-chain"}))


@pytest.mark.asyncio
async def test_invalid_provider_quote_is_rejected(settings, monkeypatch):
    monkeypatch.setattr(
        trade,
        "gateway",
        AsyncMock(
            side_effect=[
                {"provider": "Test", "options": [option()]},
                {**quote(), "pay_amount": "999"},
            ]
        ),
    )
    with pytest.raises(SpraxError, match="does not match"):
        await trade.quote(request())


@pytest.mark.asyncio
async def test_unapproved_checkout_and_tampered_token_are_rejected(settings, monkeypatch):
    monkeypatch.setattr(
        trade,
        "gateway",
        AsyncMock(
            side_effect=[
                {"provider": "Test", "options": [option()]},
                quote(),
                {"checkout_url": "https://evil.example/pay"},
            ]
        ),
    )
    result = await trade.quote(request())
    with pytest.raises(SpraxError, match="unapproved"):
        await trade.session(trade.SessionRequest(quote_token=result.data["quote_token"]))
    with pytest.raises(SpraxError, match="invalid or expired"):
        await trade.session(trade.SessionRequest(quote_token="tampered"))


@pytest.mark.parametrize("amount", ["0.0000000000000000001", "-1", "NaN", "Infinity", "1e5"])
def test_invalid_amount_syntax(amount):
    with pytest.raises(ValidationError):
        trade.QuoteRequest(
            option_id="a", amount=amount, wallet_address="sprax1testaddress", chain_id="devnet"
        )
