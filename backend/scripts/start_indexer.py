"""
Standalone indexer runner.
Usage: python scripts/start_indexer.py
"""
import asyncio

from app.core.logging import configure_logging
from app.modules.indexer.engine import IndexerEngine

configure_logging()


async def main():
    engine = IndexerEngine()
    await engine.run()


if __name__ == "__main__":
    asyncio.run(main())
