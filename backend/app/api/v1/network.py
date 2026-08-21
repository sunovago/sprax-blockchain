from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, desc
from app.core.dependencies import get_db
from app.db.models.indexer import IndexerState, NetworkStat
from app.modules.blockchain.client import get_chain_client
from app.core.response import ApiResponse
from app.core.config import settings

router = APIRouter()


@router.get("/status")
async def get_network_status(db: AsyncSession = Depends(get_db)):
    client = get_chain_client()
    import asyncio
    rpc_status, stat_r = await asyncio.gather(
        client.get_status(),
        db.execute(select(NetworkStat).order_by(desc(NetworkStat.captured_at)).limit(1)),
        return_exceptions=True,
    )
    stat = stat_r.scalar_one_or_none() if not isinstance(stat_r, Exception) else None
    rpc_ok = isinstance(rpc_status, dict)
    return ApiResponse.ok({
        "chain_id": settings.sprax_chain_id,
        "rpc_reachable": rpc_ok,
        "latest_block": rpc_status.get("latestBlockHeight", 0) if rpc_ok else 0,
        "sync_status": rpc_status.get("syncStatus") if rpc_ok else "unknown",
        "active_validators": stat.active_validators if stat else 0,
        "tps": str(stat.tps) if stat else "0",
    })
