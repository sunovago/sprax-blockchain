import asyncio

from app.core.logging import logger
from app.db.redis import redis_delete
from app.workers.celery_app import celery_app


@celery_app.task(name="app.workers.tasks.discover_refresh.refresh_discover", bind=True)
def refresh_discover(self):
    async def _run():
        await redis_delete("discover:full")
        logger.info("discover_cache_invalidated")

    try:
        asyncio.run(_run())
    except Exception as e:
        logger.error("discover_refresh_error", error=str(e))
