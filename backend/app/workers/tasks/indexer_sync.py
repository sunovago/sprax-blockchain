import asyncio
from app.workers.celery_app import celery_app
from app.core.logging import logger


@celery_app.task(name="app.workers.tasks.indexer_sync.run_indexer_cycle", bind=True, max_retries=1)
def run_indexer_cycle(self):
    """
    Runs a single indexer cycle: fetches latest blocks and indexes into PostgreSQL.
    """
    async def _run():
        from app.modules.indexer.engine import IndexerEngine
        engine = IndexerEngine()
        await engine._index_cycle()

    try:
        asyncio.run(_run())
    except Exception as e:
        logger.error("indexer_cycle_task_error", error=str(e))
