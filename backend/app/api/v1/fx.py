from fastapi import APIRouter

from app.core.response import ApiResponse
from app.modules.fx.service import get_fx_service

router = APIRouter()


@router.get("/rates")
async def get_fx_rates():
    svc = get_fx_service()
    rates = await svc.get_rates()
    return ApiResponse.ok({"base": "USD", "rates": rates})
