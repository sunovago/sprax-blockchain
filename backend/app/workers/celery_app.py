from celery import Celery
from app.core.config import settings

celery_app = Celery(
    "sprax",
    broker=settings.celery_broker_url,
    backend=settings.celery_result_backend,
    include=[
        "app.workers.tasks.market_refresh",
        "app.workers.tasks.indexer_sync",
        "app.workers.tasks.price_alerts",
        "app.workers.tasks.discover_refresh",
    ],
)

celery_app.conf.update(
    task_serializer="json",
    accept_content=["json"],
    result_serializer="json",
    timezone="UTC",
    enable_utc=True,
    task_track_started=True,
    task_acks_late=True,
    worker_prefetch_multiplier=1,
    beat_schedule={
        "refresh-markets": {
            "task": "app.workers.tasks.market_refresh.refresh_markets",
            "schedule": 60.0,  # every 60 seconds
        },
        "refresh-discover": {
            "task": "app.workers.tasks.discover_refresh.refresh_discover",
            "schedule": 120.0,
        },
        "check-price-alerts": {
            "task": "app.workers.tasks.price_alerts.check_price_alerts",
            "schedule": 30.0,
        },
        "indexer-sync": {
            "task": "app.workers.tasks.indexer_sync.run_indexer_cycle",
            "schedule": 2.0,  # every 2 seconds matches INDEXER_POLL_INTERVAL_SECONDS
        },
    },
)
