from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, desc
from pydantic import BaseModel
from app.core.dependencies import get_db, get_current_user
from app.db.models.perp import PerpMarket, Order, Position, Trade, FundingEvent
from app.core.response import ApiResponse
from app.core.feature_flags import is_enabled
from app.core.exceptions import SpraxError, PERPS_DISABLED
import uuid
from decimal import Decimal

router = APIRouter()

PERPS_GATE_MESSAGE = (
    "Perpetual futures production trading is DISABLED. "
    "Requires: approved protocol, oracle review, risk engine audit, "
    "liquidation engine audit, security audit, legal/compliance sign-off, "
    "and explicit human production approval. TESTNET/DEMO only."
)


def _require_perps():
    if not is_enabled("perps_enabled"):
        raise SpraxError(PERPS_DISABLED, PERPS_GATE_MESSAGE, 503)


# ----- PUBLIC READ APIs -----

@router.get("/markets")
async def list_perp_markets(db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(PerpMarket).where(PerpMarket.is_active == True))
    markets = result.scalars().all()
    return ApiResponse.ok([
        {
            "symbol": m.symbol,
            "base_asset": m.base_asset,
            "quote_asset": m.quote_asset,
            "max_leverage": m.max_leverage,
            "maker_fee": str(m.maker_fee),
            "taker_fee": str(m.taker_fee),
            "mark_price": str(m.mark_price),
            "index_price": str(m.index_price),
            "last_price": str(m.last_price),
            "funding_rate": str(m.funding_rate),
            "open_interest": str(m.open_interest),
            "is_testnet_only": m.is_testnet_only,
        }
        for m in markets
    ])


@router.get("/markets/{symbol}")
async def get_perp_market(symbol: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(PerpMarket).where(PerpMarket.symbol == symbol.upper()))
    market = result.scalar_one_or_none()
    if not market:
        raise HTTPException(404, "Perp market not found")
    return ApiResponse.ok({
        "symbol": market.symbol,
        "mark_price": str(market.mark_price),
        "index_price": str(market.index_price),
        "last_price": str(market.last_price),
        "funding_rate": str(market.funding_rate),
        "open_interest": str(market.open_interest),
        "max_leverage": market.max_leverage,
        "is_testnet_only": market.is_testnet_only,
    })


@router.get("/markets/{symbol}/ticker")
async def get_perp_ticker(symbol: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(PerpMarket).where(PerpMarket.symbol == symbol.upper()))
    market = result.scalar_one_or_none()
    if not market:
        raise HTTPException(404, "Perp market not found")
    return ApiResponse.ok({"mark_price": str(market.mark_price), "index_price": str(market.index_price), "last_price": str(market.last_price)})


@router.get("/markets/{symbol}/funding")
async def get_funding_history(symbol: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(
        select(FundingEvent).where(FundingEvent.symbol == symbol.upper()).order_by(desc(FundingEvent.timestamp)).limit(100)
    )
    events = result.scalars().all()
    return ApiResponse.ok([{"rate": str(e.rate), "mark_price": str(e.mark_price), "timestamp": e.timestamp.isoformat()} for e in events])


@router.get("/markets/{symbol}/trades")
async def get_recent_trades(symbol: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(
        select(Trade).where(Trade.symbol == symbol.upper()).order_by(desc(Trade.timestamp)).limit(50)
    )
    trades = result.scalars().all()
    return ApiResponse.ok([{"price": str(t.price), "size": str(t.size), "side": t.side, "timestamp": t.timestamp.isoformat()} for t in trades])


# ----- PRIVATE APIs (auth + perps_enabled gate) -----

@router.get("/positions")
async def get_positions(user: dict = Depends(get_current_user), db: AsyncSession = Depends(get_db)):
    _require_perps()
    user_id = uuid.UUID(user["sub"])
    result = await db.execute(
        select(Position).where(Position.user_id == user_id, Position.is_open == True)
    )
    positions = result.scalars().all()
    return ApiResponse.ok([
        {"id": str(p.id), "symbol": p.symbol, "side": p.side, "size": str(p.size),
         "entry_price": str(p.entry_price), "mark_price": str(p.mark_price),
         "leverage": p.leverage, "margin": str(p.margin),
         "liquidation_price": str(p.liquidation_price), "unrealized_pnl": str(p.unrealized_pnl),
         "is_testnet": p.is_testnet}
        for p in positions
    ])


class OrderRequest(BaseModel):
    symbol: str
    side: str
    order_type: str
    size: float
    price: float | None = None
    leverage: int = 1


@router.post("/orders")
async def place_order(
    req: OrderRequest,
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    """TESTNET/DEMO ONLY. Production Perps BLOCKED."""
    _require_perps()
    # This endpoint only operates in testnet mode
    user_id = uuid.UUID(user["sub"])
    order = Order(
        user_id=user_id,
        symbol=req.symbol.upper(),
        side=req.side,
        order_type=req.order_type,
        size=Decimal(str(req.size)),
        price=Decimal(str(req.price)) if req.price else None,
        leverage=req.leverage,
        is_testnet=True,  # ALWAYS testnet until production approval
    )
    db.add(order)
    await db.commit()
    return ApiResponse.ok({"id": str(order.id), "status": "open", "is_testnet": True})
