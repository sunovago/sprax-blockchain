from fastapi import APIRouter, Query
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, or_
from fastapi import Depends
from app.core.dependencies import get_db
from app.db.models.blockchain import Block, Transaction, Address
from app.db.models.validator import Validator
from app.modules.markets.service import get_market_service
from app.modules.blockchain.address import (
    is_valid_address, is_valid_tx_hash, is_block_height, is_valid_valoper
)
from app.core.response import ApiResponse

router = APIRouter()


@router.get("")
async def search(
    q: str = Query(..., min_length=1, max_length=200),
    db: AsyncSession = Depends(get_db),
):
    q = q.strip()
    results = []

    # 1. Block height
    if is_block_height(q):
        r = await db.execute(select(Block).where(Block.height == int(q)))
        block = r.scalar_one_or_none()
        if block:
            results.append({"type": "block", "data": {"height": block.height, "hash": block.hash}})

    # 2. Transaction hash
    if is_valid_tx_hash(q):
        r = await db.execute(select(Transaction).where(Transaction.hash == q))
        tx = r.scalar_one_or_none()
        if tx:
            results.append({"type": "transaction", "data": {"hash": tx.hash, "status": tx.status}})

    # 3. Block hash (if starts with 0x and 66 chars or 64 hex)
    if not results and (q.startswith("0x") or (len(q) == 64 and all(c in "0123456789abcdefABCDEF" for c in q))):
        r = await db.execute(select(Block).where(Block.hash == q))
        block = r.scalar_one_or_none()
        if block:
            results.append({"type": "block", "data": {"height": block.height, "hash": block.hash}})

    # 4. Address
    if is_valid_address(q):
        r = await db.execute(select(Address).where(Address.address == q))
        addr = r.scalar_one_or_none()
        if addr:
            results.append({"type": "address", "data": {"address": addr.address, "tx_count": addr.tx_count}})
        else:
            results.append({"type": "address", "data": {"address": q, "tx_count": 0, "indexed": False}})

    # 5. Validator operator address
    if is_valid_valoper(q):
        r = await db.execute(select(Validator).where(Validator.operator_address == q))
        val = r.scalar_one_or_none()
        if val:
            results.append({"type": "validator", "data": {"operator_address": val.operator_address, "moniker": val.moniker}})

    # 6. Validator moniker search
    if not results:
        r = await db.execute(
            select(Validator).where(Validator.moniker.ilike(f"%{q}%")).limit(5)
        )
        vals = r.scalars().all()
        for val in vals:
            results.append({"type": "validator", "data": {"operator_address": val.operator_address, "moniker": val.moniker}})

    # 7. Market / asset
    if not results or len(results) < 5:
        try:
            market_svc = get_market_service()
            markets = await market_svc.get_markets()
            for m in markets:
                if q.upper() in (m.get("symbol", "").upper(), m.get("name", "").upper()):
                    results.append({"type": "asset", "data": m})
                    break
        except Exception:
            pass

    return ApiResponse.ok({"query": q, "results": results})
