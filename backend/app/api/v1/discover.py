from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, desc
from app.core.dependencies import get_db
from app.db.models.discover import EcosystemProject
from app.db.models.indexer import NetworkStat, IndexerState
from app.db.models.validator import Validator
from app.modules.markets.service import get_market_service
from app.core.response import ApiResponse
from app.core.feature_flags import is_enabled
from app.core.exceptions import SpraxError
from app.db.redis import redis_get, redis_set, TTL_DISCOVER

router = APIRouter()


@router.get("")
async def get_discover(db: AsyncSession = Depends(get_db)):
    if not is_enabled("discover_enabled"):
        raise SpraxError("DISCOVER_DISABLED", "Discover feature is disabled", 503)

    cached = await redis_get("discover:full")
    if cached:
        return ApiResponse.ok(cached)

    market_svc = get_market_service()

    # Gather data concurrently
    import asyncio
    gainers, losers, trending, volume_leaders, markets = await asyncio.gather(
        market_svc.get_gainers(5),
        market_svc.get_losers(5),
        market_svc.get_trending(),
        market_svc.get_volume_leaders(5),
        market_svc.get_markets(),
        return_exceptions=True,
    )

    # Projects from DB
    proj_r = await db.execute(
        select(EcosystemProject)
        .where(EcosystemProject.is_active == True)
        .order_by(EcosystemProject.sort_order)
    )
    projects = proj_r.scalars().all()

    # Validators
    val_r = await db.execute(
        select(Validator).where(Validator.status == "ACTIVE").order_by(desc(Validator.voting_power)).limit(10)
    )
    validators = val_r.scalars().all()

    # Network stats
    stat_r = await db.execute(
        select(NetworkStat).order_by(desc(NetworkStat.captured_at)).limit(1)
    )
    stat = stat_r.scalar_one_or_none()

    data = {
        "trending": trending if isinstance(trending, list) else [],
        "gainers": gainers if isinstance(gainers, list) else [],
        "losers": losers if isinstance(losers, list) else [],
        "volume_leaders": volume_leaders if isinstance(volume_leaders, list) else [],
        "sprax_ecosystem": [
            {
                "id": p.project_id,
                "name": p.name,
                "tag": p.tag,
                "description": p.description,
                "icon_emoji": p.icon_emoji,
                "category": p.category,
                "website_url": p.website_url,
                "tvl_usd": str(p.tvl_usd) if p.tvl_usd else None,
                "active_users_24h": p.active_users_24h,
                "is_verified": p.is_verified,
            }
            for p in projects
        ],
        "validators": [
            {
                "operator_address": v.operator_address,
                "moniker": v.moniker,
                "voting_power": v.voting_power,
                "commission_rate": str(v.commission_rate),
                "status": v.status,
                "uptime_percent": str(v.uptime_percent),
            }
            for v in validators
        ],
        "network_highlights": {
            "active_validators": stat.active_validators if stat else 0,
            "tps": str(stat.tps) if stat else "0",
            "avg_block_time_ms": stat.avg_block_time_ms if stat else 2000,
            "total_transactions": stat.total_transactions if stat else 0,
        },
    }

    await redis_set("discover:full", data, TTL_DISCOVER)
    return ApiResponse.ok(data)
