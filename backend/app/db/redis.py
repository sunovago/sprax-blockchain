import redis.asyncio as aioredis
from app.core.config import settings
from typing import Any
import json

_redis: aioredis.Redis | None = None


async def get_redis() -> aioredis.Redis:
    global _redis
    if _redis is None:
        _redis = aioredis.from_url(
            settings.redis_url,
            encoding="utf-8",
            decode_responses=True,
        )
    return _redis


async def redis_get(key: str) -> Any | None:
    r = await get_redis()
    val = await r.get(key)
    if val is None:
        return None
    try:
        return json.loads(val)
    except (json.JSONDecodeError, TypeError):
        return val


async def redis_set(key: str, value: Any, ttl: int | None = None) -> None:
    r = await get_redis()
    encoded = json.dumps(value) if not isinstance(value, str) else value
    if ttl:
        await r.setex(key, ttl, encoded)
    else:
        await r.set(key, encoded)


async def redis_delete(key: str) -> None:
    r = await get_redis()
    await r.delete(key)


async def redis_publish(channel: str, message: Any) -> None:
    r = await get_redis()
    await r.publish(channel, json.dumps(message))


# TTL constants (seconds)
TTL_TICKER = 15
TTL_MARKETS = 60
TTL_TRENDING = 300
TTL_CANDLES_1H = 300
TTL_FX = 3600
TTL_VALIDATORS = 60
TTL_NETWORK_STATS = 30
TTL_DISCOVER = 120
TTL_SEARCH = 60
TTL_CONFIG = 600
