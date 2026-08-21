from fastapi import APIRouter, HTTPException, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, desc
from app.core.dependencies import get_db
from app.db.models.validator import Validator, Delegation
from app.core.response import ApiResponse

router = APIRouter()


@router.get("")
async def list_validators(db: AsyncSession = Depends(get_db)):
    result = await db.execute(
        select(Validator).order_by(desc(Validator.voting_power))
    )
    validators = result.scalars().all()
    return ApiResponse.ok([
        {
            "operator_address": v.operator_address,
            "moniker": v.moniker,
            "identity": v.identity,
            "website": v.website,
            "details": v.details,
            "commission_rate": str(v.commission_rate),
            "commission_max_rate": str(v.commission_max_rate),
            "bonded_tokens_atto": str(v.bonded_tokens),
            "voting_power": v.voting_power,
            "voting_power_share_pct": str(v.voting_power_share),
            "status": v.status,
            "jailed": v.jailed,
            "uptime_percent": str(v.uptime_percent),
            "missed_blocks": v.missed_blocks,
            "slash_count": v.slash_count,
            "delegator_count": v.delegator_count,
        }
        for v in validators
    ])


@router.get("/{operator_address}")
async def get_validator(operator_address: str, db: AsyncSession = Depends(get_db)):
    result = await db.execute(
        select(Validator).where(Validator.operator_address == operator_address)
    )
    val = result.scalar_one_or_none()
    if not val:
        raise HTTPException(404, "Validator not found")
    # Delegations
    d_result = await db.execute(
        select(Delegation).where(Delegation.validator_address == operator_address)
    )
    delegations = d_result.scalars().all()
    return ApiResponse.ok({
        "operator_address": val.operator_address,
        "moniker": val.moniker,
        "identity": val.identity,
        "website": val.website,
        "details": val.details,
        "commission_rate": str(val.commission_rate),
        "commission_max_rate": str(val.commission_max_rate),
        "bonded_tokens_atto": str(val.bonded_tokens),
        "voting_power": val.voting_power,
        "voting_power_share_pct": str(val.voting_power_share),
        "status": val.status,
        "jailed": val.jailed,
        "uptime_percent": str(val.uptime_percent),
        "missed_blocks": val.missed_blocks,
        "slash_count": val.slash_count,
        "delegator_count": val.delegator_count,
        "delegations": [
            {"delegator": d.delegator_address, "amount_atto": str(d.amount), "shares": str(d.shares)}
            for d in delegations
        ],
    })
