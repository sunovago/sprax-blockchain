"""Quotes and hosted checkout only; this API never executes a trade or holds keys."""

import hashlib
from datetime import UTC, datetime
from decimal import Decimal
from typing import Literal
from urllib.parse import urlparse

import httpx
from fastapi import APIRouter
from jose import JWTError, jwt
from pydantic import BaseModel, Field, ValidationError, field_validator

from app.core.config import get_settings
from app.core.exceptions import SpraxError
from app.core.response import ApiResponse

router = APIRouter()


class TradeOption(BaseModel):
    id: str = Field(min_length=1, max_length=120)
    label: str = Field(min_length=1, max_length=160)
    mode: Literal["fiat", "spot"]
    side: Literal["buy", "sell"]
    asset: str = Field(min_length=1, max_length=20)
    payment_asset: str = Field(min_length=1, max_length=20)
    payment_method: str = Field(min_length=1, max_length=60)
    chain_id: str = Field(min_length=1, max_length=100)
    min_amount: Decimal = Field(gt=0, allow_inf_nan=False)
    max_amount: Decimal = Field(gt=0, allow_inf_nan=False)


class Capabilities(BaseModel):
    provider: str
    options: list[TradeOption]


class QuoteRequest(BaseModel):
    option_id: str = Field(min_length=1, max_length=120)
    amount: str = Field(pattern=r"^[0-9]{1,18}(\.[0-9]{1,18})?$", max_length=37)
    wallet_address: str = Field(min_length=10, max_length=160, pattern=r"^[a-zA-Z0-9]+$")
    chain_id: str = Field(min_length=1, max_length=100)


class GatewayQuote(BaseModel):
    quote_id: str = Field(min_length=1, max_length=200)
    provider: str = Field(min_length=1, max_length=160)
    pay_amount: Decimal = Field(gt=0, allow_inf_nan=False)
    pay_asset: str
    receive_amount: Decimal = Field(gt=0, allow_inf_nan=False)
    receive_asset: str
    fee_amount: Decimal = Field(ge=0, allow_inf_nan=False)
    fee_asset: str
    expires_at: datetime

    @field_validator("expires_at")
    @classmethod
    def require_timezone(cls, value):
        if value.tzinfo is None:
            raise ValueError("Quote expiry must include timezone")
        return value


class SessionRequest(BaseModel):
    quote_token: str = Field(min_length=1, max_length=8000)


def unavailable():
    return SpraxError(
        "TRADE_UNAVAILABLE", "Buy/Sell provider is not configured for this network.", 503
    )


async def gateway(path: str, body: dict | None = None, idempotency: str | None = None):
    settings = get_settings()
    base = urlparse(settings.trade_gateway_url)
    if base.scheme != "https" or not base.hostname or not settings.trade_gateway_token:
        raise unavailable()
    headers = {"Authorization": f"Bearer {settings.trade_gateway_token}"}
    if idempotency:
        headers["Idempotency-Key"] = idempotency
    try:
        async with httpx.AsyncClient(timeout=15, follow_redirects=False) as client:
            url = settings.trade_gateway_url.rstrip("/") + path
            result = (
                await client.get(url, headers=headers)
                if body is None
                else await client.post(url, json=body, headers=headers)
            )
            result.raise_for_status()
            return result.json()
    except (httpx.HTTPError, ValueError) as exc:
        raise SpraxError(
            "TRADE_PROVIDER_UNAVAILABLE",
            "Provider could not complete this request. Please retry.",
            502,
        ) from exc


async def configured_capabilities() -> Capabilities:
    try:
        return Capabilities.model_validate(await gateway("/capabilities"))
    except ValidationError as exc:
        raise SpraxError(
            "INVALID_TRADE_PROVIDER", "Provider returned invalid supported markets.", 502
        ) from exc


@router.get("/capabilities")
async def capabilities():
    settings = get_settings()
    if (
        not settings.trade_gateway_url
        or not settings.trade_gateway_token
        or not settings.trade_checkout_hosts
    ):
        return ApiResponse.ok(
            {
                "provider": "",
                "options": [],
                "message": "Fiat and Spot Buy/Sell will be available when a provider is connected.",
            }
        )
    data = await configured_capabilities()
    return ApiResponse.ok(data.model_dump(mode="json"))


@router.post("/quotes")
async def quote(request: QuoteRequest):
    settings = get_settings()
    if not settings.trade_checkout_hosts or settings.secret_key == "change-this-in-production":
        raise unavailable()
    available = await configured_capabilities()
    option = next(
        (
            item
            for item in available.options
            if item.id == request.option_id and item.chain_id == request.chain_id
        ),
        None,
    )
    amount = Decimal(request.amount)
    if option is None:
        raise SpraxError(
            "TRADE_NOT_SUPPORTED",
            "This pair or payment method is unavailable on the selected network.",
            400,
        )
    if not option.min_amount <= amount <= option.max_amount:
        raise SpraxError(
            "TRADE_AMOUNT_INVALID",
            f"Enter an amount between {option.min_amount} and {option.max_amount}.",
            400,
        )
    try:
        result = GatewayQuote.model_validate(await gateway("/quotes", request.model_dump()))
    except ValidationError as exc:
        raise SpraxError("INVALID_TRADE_QUOTE", "Provider returned an invalid quote.", 502) from exc
    pay_asset = option.payment_asset if option.side == "buy" else option.asset
    receive_asset = option.asset if option.side == "buy" else option.payment_asset
    if (
        result.pay_amount != amount
        or result.pay_asset != pay_asset
        or result.receive_asset != receive_asset
    ):
        raise SpraxError(
            "INVALID_TRADE_QUOTE", "Provider quote does not match the requested trade.", 502
        )
    now = datetime.now(UTC)
    if not 0 < (result.expires_at - now).total_seconds() <= 900:
        raise SpraxError("QUOTE_EXPIRED", "Quote expired. Request a new quote.", 400)
    payload = {
        "purpose": "trade_checkout",
        "exp": int(result.expires_at.timestamp()),
        "request": request.model_dump(),
        "quote_id": result.quote_id,
    }
    token = jwt.encode(payload, settings.secret_key, algorithm="HS256")
    return ApiResponse.ok({**result.model_dump(mode="json"), "quote_token": token})


@router.post("/sessions")
async def session(request: SessionRequest):
    settings = get_settings()
    if settings.secret_key == "change-this-in-production" or not settings.trade_checkout_hosts:
        raise unavailable()
    try:
        payload = jwt.decode(
            request.quote_token,
            settings.secret_key,
            algorithms=["HS256"],
            options={"require_exp": True},
        )
        if (
            payload.get("purpose") != "trade_checkout"
            or not payload.get("quote_id")
            or not isinstance(payload.get("request"), dict)
        ):
            raise JWTError("Wrong token purpose")
    except JWTError as exc:
        raise SpraxError(
            "QUOTE_EXPIRED", "Quote is invalid or expired. Request a new quote.", 400
        ) from exc
    # The same reviewed quote always creates/resumes the same hosted session.
    key = hashlib.sha256(request.quote_token.encode()).hexdigest()
    result = await gateway(
        "/sessions", {**payload["request"], "quote_id": payload["quote_id"]}, key
    )
    url = result.get("checkout_url", "") if isinstance(result, dict) else ""
    try:
        parsed = urlparse(url if isinstance(url, str) else "")
        port = parsed.port
    except ValueError as exc:
        raise SpraxError(
            "INVALID_CHECKOUT", "Provider returned an invalid checkout address.", 502
        ) from exc
    hosts = {
        host.strip().lower() for host in settings.trade_checkout_hosts.split(",") if host.strip()
    }
    if (
        parsed.scheme != "https"
        or parsed.hostname not in hosts
        or parsed.username
        or parsed.password
        or port not in (None, 443)
    ):
        raise SpraxError(
            "INVALID_CHECKOUT", "Provider returned an unapproved checkout address.", 502
        )
    return ApiResponse.ok({"checkout_url": url, "status": "awaiting_provider_confirmation"})
