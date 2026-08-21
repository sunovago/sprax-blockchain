from fastapi import APIRouter, Query, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, func, desc
from fastapi import Depends
from app.core.dependencies import get_db
from app.db.models.blockchain import Block, Transaction, Address
from app.db.models.validator import Validator
from app.db.models.indexer import IndexerState, NetworkStat
from app.core.response import ApiResponse, PaginatedResponse
from app.modules.blockchain.address import atto_to_sprx
from app.core.exceptions import SpraxError, INVALID_ADDRESS

router = APIRouter()

MAX_LIMIT = 100


@router.get("/blocks")
async def list_blocks(
    limit: int = Query(20, ge=1, le=MAX_LIMIT),
    offset: int = Query(0, ge=0),
    db: AsyncSession = Depends(get_db),
):
    total_r = await db.execute(select(func.count()).select_from(Block))
    total = total_r.scalar_one()
    result = await db.execute(
        select(Block).order_by(desc(Block.height)).offset(offset).limit(limit)
    )
    blocks = result.scalars().all()
    data = [
        {
            "height": b.height,
            "hash": b.hash,
            "proposer": b.proposer,
            "tx_count": b.tx_count,
            "gas_used": b.gas_used,
            "timestamp": b.timestamp.isoformat() if b.timestamp else None,
        }
        for b in blocks
    ]
    return PaginatedResponse.ok(data, total, limit, offset)


@router.get("/blocks/{height_or_hash}")
async def get_block(height_or_hash: str, db: AsyncSession = Depends(get_db)):
    if height_or_hash.isdigit():
        result = await db.execute(select(Block).where(Block.height == int(height_or_hash)))
    else:
        result = await db.execute(select(Block).where(Block.hash == height_or_hash))
    block = result.scalar_one_or_none()
    if not block:
        raise HTTPException(404, "Block not found")
    txs_result = await db.execute(
        select(Transaction).where(Transaction.block_height == block.height)
    )
    txs = txs_result.scalars().all()
    return ApiResponse.ok({
        "height": block.height,
        "hash": block.hash,
        "parent_hash": block.parent_hash,
        "proposer": block.proposer,
        "state_root": block.state_root,
        "tx_count": block.tx_count,
        "gas_used": block.gas_used,
        "timestamp": block.timestamp.isoformat() if block.timestamp else None,
        "transactions": [t.hash for t in txs],
    })


@router.get("/transactions")
async def list_transactions(
    limit: int = Query(20, ge=1, le=MAX_LIMIT),
    offset: int = Query(0, ge=0),
    db: AsyncSession = Depends(get_db),
):
    total_r = await db.execute(select(func.count()).select_from(Transaction))
    total = total_r.scalar_one()
    result = await db.execute(
        select(Transaction).order_by(desc(Transaction.timestamp)).offset(offset).limit(limit)
    )
    txs = result.scalars().all()
    return PaginatedResponse.ok([_tx_dict(t) for t in txs], total, limit, offset)


@router.get("/transactions/{hash}")
async def get_transaction(hash: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(Transaction).where(Transaction.hash == hash))
    tx = result.scalar_one_or_none()
    if not tx:
        raise HTTPException(404, "Transaction not found")
    return ApiResponse.ok(_tx_dict(tx))


@router.get("/addresses/{address}")
async def get_address(address: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(select(Address).where(Address.address == address))
    addr = result.scalar_one_or_none()
    if not addr:
        raise HTTPException(404, "Address not found in index")
    return ApiResponse.ok({
        "address": addr.address,
        "balance_atto": str(addr.balance),
        "balance_sprx": atto_to_sprx(str(addr.balance)),
        "nonce": addr.nonce,
        "tx_count": addr.tx_count,
        "first_seen_height": addr.first_seen_height,
        "last_seen_height": addr.last_seen_height,
    })


@router.get("/addresses/{address}/transactions")
async def get_address_transactions(
    address: str,
    limit: int = Query(20, ge=1, le=MAX_LIMIT),
    offset: int = Query(0, ge=0),
    db: AsyncSession = Depends(get_db),
):
    from sqlalchemy import or_
    total_r = await db.execute(
        select(func.count()).select_from(Transaction).where(
            or_(Transaction.sender == address, Transaction.recipient == address)
        )
    )
    total = total_r.scalar_one()
    result = await db.execute(
        select(Transaction)
        .where(or_(Transaction.sender == address, Transaction.recipient == address))
        .order_by(desc(Transaction.timestamp))
        .offset(offset)
        .limit(limit)
    )
    txs = result.scalars().all()
    return PaginatedResponse.ok([_tx_dict(t) for t in txs], total, limit, offset)


@router.get("/validators")
async def list_validators(db: AsyncSession = Depends(get_db)):
    result = await db.execute(
        select(Validator).order_by(desc(Validator.voting_power))
    )
    validators = result.scalars().all()
    return ApiResponse.ok([_val_dict(v) for v in validators])


@router.get("/validators/{operator_address}")
async def get_validator(operator_address: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(
        select(Validator).where(Validator.operator_address == operator_address)
    )
    val = result.scalar_one_or_none()
    if not val:
        raise HTTPException(404, "Validator not found")
    return ApiResponse.ok(_val_dict(val))


@router.get("/network")
async def get_network(db: AsyncSession = Depends(get_db)):
    state_r = await db.execute(select(IndexerState).where(IndexerState.id == 1))
    state = state_r.scalar_one_or_none()
    stat_r = await db.execute(
        select(NetworkStat).order_by(desc(NetworkStat.captured_at)).limit(1)
    )
    stat = stat_r.scalar_one_or_none()
    return ApiResponse.ok({
        "latest_chain_height": state.latest_chain_height if state else 0,
        "latest_indexed_height": state.latest_indexed_height if state else 0,
        "lag_blocks": state.lag_blocks if state else 0,
        "active_validators": stat.active_validators if stat else 0,
        "tps": str(stat.tps) if stat else "0",
        "avg_block_time_ms": stat.avg_block_time_ms if stat else 2000,
        "total_transactions": stat.total_transactions if stat else 0,
    })


def _tx_dict(t: Transaction) -> dict:
    return {
        "hash": t.hash,
        "block_height": t.block_height,
        "sender": t.sender,
        "recipient": t.recipient,
        "amount_atto": str(t.amount),
        "amount_sprx": atto_to_sprx(str(t.amount)),
        "fee_atto": str(t.fee),
        "gas_used": t.gas_used,
        "nonce": t.nonce,
        "memo": t.memo,
        "type": t.tx_type,
        "status": t.status,
        "timestamp": t.timestamp.isoformat() if t.timestamp else None,
    }


def _val_dict(v: Validator) -> dict:
    return {
        "operator_address": v.operator_address,
        "moniker": v.moniker,
        "identity": v.identity,
        "website": v.website,
        "commission_rate": str(v.commission_rate),
        "bonded_tokens_atto": str(v.bonded_tokens),
        "voting_power": v.voting_power,
        "voting_power_share_pct": str(v.voting_power_share),
        "status": v.status,
        "jailed": v.jailed,
        "uptime_percent": str(v.uptime_percent),
        "missed_blocks": v.missed_blocks,
        "delegator_count": v.delegator_count,
    }
