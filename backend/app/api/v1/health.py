from fastapi import APIRouter
from app.db.session import engine
from app.db.redis import get_redis
from app.modules.blockchain.client import get_chain_client
import asyncio

router = APIRouter()


@router.get("/")
async def health():
    return {"status": "ok", "service": "sprax-backend"}


@router.get("/ready")
async def ready():
    checks = {}
    # DB check
    try:
        async with engine.connect() as conn:
            await conn.execute(__import__("sqlalchemy").text("SELECT 1"))
        checks["postgres"] = "ok"
    except Exception as e:
        checks["postgres"] = f"error: {e}"

    # Redis check
    try:
        r = await get_redis()
        await r.ping()
        checks["redis"] = "ok"
    except Exception as e:
        checks["redis"] = f"error: {e}"

    # RPC check
    try:
        client = get_chain_client()
        status = await asyncio.wait_for(client.get_status(), timeout=3.0)
        checks["sprax_rpc"] = "ok" if status else "no_response"
    except Exception as e:
        checks["sprax_rpc"] = f"error: {e}"

    all_ok = all(v == "ok" for v in checks.values())
    return {
        "ready": all_ok,
        "checks": checks,
    }
