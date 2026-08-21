from fastapi import APIRouter, Depends, Query
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, desc, func
from app.core.dependencies import get_db, get_current_user
from app.db.models.notification import Notification, PriceAlert
from app.core.response import ApiResponse, PaginatedResponse
from app.core.feature_flags import is_enabled
from app.core.exceptions import SpraxError
from decimal import Decimal
import uuid

router = APIRouter()


class PriceAlertRequest(BaseModel):
    symbol: str
    condition: str  # above, below
    target_price_usd: float


@router.get("")
async def list_notifications(
    limit: int = Query(20, ge=1, le=100),
    offset: int = Query(0, ge=0),
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    user_id = uuid.UUID(user["sub"])
    total_r = await db.execute(
        select(func.count()).select_from(Notification).where(Notification.user_id == user_id)
    )
    total = total_r.scalar_one()
    result = await db.execute(
        select(Notification)
        .where(Notification.user_id == user_id)
        .order_by(desc(Notification.created_at))
        .offset(offset)
        .limit(limit)
    )
    notifs = result.scalars().all()
    return PaginatedResponse.ok(
        [
            {
                "id": str(n.id),
                "type": n.type,
                "title": n.title,
                "body": n.body,
                "is_read": n.is_read,
                "created_at": n.created_at.isoformat(),
            }
            for n in notifs
        ],
        total, limit, offset
    )


@router.patch("/{notification_id}/read")
async def mark_read(
    notification_id: str,
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    user_id = uuid.UUID(user["sub"])
    nid = uuid.UUID(notification_id)
    result = await db.execute(
        select(Notification).where(Notification.id == nid, Notification.user_id == user_id)
    )
    notif = result.scalar_one_or_none()
    if not notif:
        raise SpraxError("NOT_FOUND", "Notification not found", 404)
    notif.is_read = True
    await db.commit()
    return ApiResponse.ok({"read": True})


@router.post("/alerts/price")
async def create_price_alert(
    req: PriceAlertRequest,
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    if not is_enabled("price_alerts_enabled"):
        raise SpraxError("PRICE_ALERTS_DISABLED", "Price alerts are disabled", 503)
    if req.condition not in ("above", "below"):
        raise SpraxError("INVALID_REQUEST", "Condition must be 'above' or 'below'", 400)
    user_id = uuid.UUID(user["sub"])
    alert = PriceAlert(
        user_id=user_id,
        symbol=req.symbol.upper(),
        condition=req.condition,
        target_price_usd=Decimal(str(req.target_price_usd)),
    )
    db.add(alert)
    await db.commit()
    return ApiResponse.ok({"id": str(alert.id), "symbol": alert.symbol, "condition": alert.condition})


@router.get("/alerts/price")
async def list_price_alerts(
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    user_id = uuid.UUID(user["sub"])
    result = await db.execute(
        select(PriceAlert).where(PriceAlert.user_id == user_id, PriceAlert.is_active == True)
    )
    alerts = result.scalars().all()
    return ApiResponse.ok([
        {"id": str(a.id), "symbol": a.symbol, "condition": a.condition, "target_price_usd": str(a.target_price_usd)}
        for a in alerts
    ])


@router.delete("/alerts/price/{alert_id}")
async def delete_price_alert(
    alert_id: str,
    user: dict = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
):
    user_id = uuid.UUID(user["sub"])
    result = await db.execute(
        select(PriceAlert).where(PriceAlert.id == uuid.UUID(alert_id), PriceAlert.user_id == user_id)
    )
    alert = result.scalar_one_or_none()
    if not alert:
        raise SpraxError("NOT_FOUND", "Price alert not found", 404)
    alert.is_active = False
    await db.commit()
    return ApiResponse.ok({"deleted": True})
