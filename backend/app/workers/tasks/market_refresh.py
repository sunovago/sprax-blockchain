import asyncio
from app.workers.celery_app import celery_app
from app.modules.markets.service import get_market_service
from app.db.redis import redis_delete
from app.core.logging import logger


@celery_app.task(name="app.workers.tasks.market_refresh.refresh_markets", bind=True, max_retries=3)
def refresh_markets(self):
    """
    Periodically fetch fresh market data from provider and update Redis cache.
    """
    async def _run():
        svc = get_market_service()
        await redis_delete("markets:all")
        await redis_delete("markets:trending")
        markets = await svc.get_markets()
        logger.info("market_refresh_done", count=len(markets))

    try:
        asyncio.run(_run())
    except Exception as e:
        logger.error("market_refresh_error", error=str(e))
        raise self.retry(exc=e, countdown=30)
