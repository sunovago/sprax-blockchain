from fastapi import APIRouter
from app.core.feature_flags import get_public_flags
from app.core.config import settings
from app.core.response import ApiResponse

router = APIRouter()


@router.get("/public")
async def get_public_config():
    """Returns safe public configuration for Flutter clients. Never exposes secrets."""
    return ApiResponse.ok({
        "network": {
            "name": settings.app_env,
            "chain_id": settings.sprax_chain_id,
            "rpc_url": settings.sprax_rpc_url,
            "explorer_url": "https://explorer.sprax.network",
            "address_prefix": settings.sprax_address_prefix,
            "decimals": settings.sprax_decimals,
            "default_fee_atto": settings.sprax_default_fee_atto,
            "default_gas": settings.sprax_default_gas,
        },
        "feature_flags": get_public_flags(),
        "supported_currencies": ["USD", "INR", "EUR", "GBP", "JPY"],
        "app_min_version": "1.0.0",
    })
