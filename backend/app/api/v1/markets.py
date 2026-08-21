from fastapi import APIRouter, Query, HTTPException
from app.modules.markets.service import get_market_service
from app.modules.fx.service import get_fx_service
from app.core.response import ApiResponse
from app.core.feature_flags import is_enabled
from app.core.exceptions import SpraxError, MARKET_NOT_FOUND

router = APIRouter()


def _check_markets():
    if not is_enabled("markets_enabled"):
        raise SpraxError("MARKETS_DISABLED", "Markets feature is disabled", 503)


@router.get("")
async def list_markets():
    _check_markets()
    svc = get_market_service()
    return ApiResponse.ok(await svc.get_markets())


@router.get("/gainers")
async def get_gainers(limit: int = Query(10, ge=1, le=50)):
    _check_markets()
    svc = get_market_service()
    return ApiResponse.ok(await svc.get_gainers(limit))


@router.get("/losers")
async def get_losers(limit: int = Query(10, ge=1, le=50)):
    _check_markets()
    svc = get_market_service()
    return ApiResponse.ok(await svc.get_losers(limit))


@router.get("/trending")
async def get_trending():
    _check_markets()
    svc = get_market_service()
    return ApiResponse.ok(await svc.get_trending())


@router.get("/volume-leaders")
async def get_volume_leaders(limit: int = Query(10, ge=1, le=50)):
    _check_markets()
    svc = get_market_service()
    return ApiResponse.ok(await svc.get_volume_leaders(limit))


@router.get("/{symbol}")
async def get_market(symbol: str):
    _check_markets()
    svc = get_market_service()
    ticker = await svc.get_ticker(symbol)
    if not ticker:
        raise SpraxError(MARKET_NOT_FOUND, f"Market not found: {symbol}", 404)
    return ApiResponse.ok(ticker)


@router.get("/{symbol}/ticker")
async def get_ticker(symbol: str):
    _check_markets()
    svc = get_market_service()
    ticker = await svc.get_ticker(symbol)
    if not ticker:
        raise SpraxError(MARKET_NOT_FOUND, f"Ticker not found: {symbol}", 404)
    return ApiResponse.ok(ticker)


@router.get("/{symbol}/candles")
async def get_candles(
    symbol: str,
    interval: str = Query("1h", pattern="^(1m|5m|15m|1h|4h|1d)$"),
    limit: int = Query(100, ge=1, le=500),
):
    _check_markets()
    svc = get_market_service()
    candles = await svc.get_candles(symbol, interval, limit)
    return ApiResponse.ok(candles)


@router.get("/{symbol}/value")
async def get_value_in_currency(
    symbol: str,
    currency: str = Query("USD"),
    amount: float = Query(1.0),
):
    """Convert an asset amount to a local fiat currency."""
    _check_markets()
    currency = currency.upper()
    valid_currencies = ["USD", "INR", "EUR", "GBP", "JPY"]
    if currency not in valid_currencies:
        raise SpraxError("INVALID_CURRENCY", f"Unsupported currency: {currency}", 400)
    market_svc = get_market_service()
    fx_svc = get_fx_service()
    ticker = await market_svc.get_ticker(symbol)
    if not ticker:
        raise SpraxError(MARKET_NOT_FOUND, f"Market not found: {symbol}", 404)
    price_usd = ticker.get("price_usd") or 0
    value_usd = float(price_usd) * amount
    value_fiat = await fx_svc.convert(value_usd, currency)
    return ApiResponse.ok({
        "symbol": symbol.upper(),
        "amount": amount,
        "price_usd": price_usd,
        "currency": currency,
        "value": value_fiat,
    })
