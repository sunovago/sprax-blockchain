import asyncio
from app.workers.celery_app import celery_app
from app.core.logging import logger


@celery_app.task(name="app.workers.tasks.price_alerts.check_price_alerts", bind=True)
def check_price_alerts(self):
    """Check active price alerts against current market prices."""
    async def _run():
        from sqlalchemy import select
        from app.db.session import async_session_factory
        from app.db.models.notification import PriceAlert, Notification
        from app.modules.markets.service import get_market_service
        from datetime import datetime, timezone

        svc = get_market_service()
        markets = await svc.get_markets()
        price_map = {m["symbol"]: m.get("price_usd") for m in markets if m.get("price_usd")}

        async with async_session_factory() as db:
            result = await db.execute(
                select(PriceAlert).where(PriceAlert.is_active == True, PriceAlert.triggered_at == None)
            )
            alerts = result.scalars().all()

            for alert in alerts:
                current = price_map.get(alert.symbol.upper())
                if current is None:
                    continue
                current_f = float(current)
                target_f = float(alert.target_price_usd)
                triggered = (
                    (alert.condition == "above" and current_f >= target_f) or
                    (alert.condition == "below" and current_f <= target_f)
                )
                if triggered:
                    alert.triggered_at = datetime.now(timezone.utc)
                    alert.is_active = False
                    notif = Notification(
                        user_id=alert.user_id,
                        type="price_alert",
                        title=f"{alert.symbol} price alert triggered",
                        body=f"{alert.symbol} is {alert.condition} ${target_f:.4f}. Current: ${current_f:.4f}",
                        metadata={"symbol": alert.symbol, "condition": alert.condition, "target": str(target_f), "current": str(current_f)},
                    )
                    db.add(notif)
            await db.commit()
        logger.info("price_alerts_checked", total=len(alerts))

    try:
        asyncio.run(_run())
    except Exception as e:
        logger.error("price_alerts_error", error=str(e))
