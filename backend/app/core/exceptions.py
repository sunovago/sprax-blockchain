import uuid
from typing import Any

from fastapi import HTTPException, Request
from fastapi.responses import JSONResponse


class SpraxError(Exception):
    def __init__(self, code: str, message: str, status_code: int = 400, detail: Any = None):
        self.code = code
        self.message = message
        self.status_code = status_code
        self.detail = detail
        super().__init__(message)


# Error code constants
INVALID_REQUEST = "INVALID_REQUEST"
UNAUTHORIZED = "UNAUTHORIZED"
FORBIDDEN = "FORBIDDEN"
NOT_FOUND = "NOT_FOUND"
RATE_LIMITED = "RATE_LIMITED"
RPC_UNAVAILABLE = "RPC_UNAVAILABLE"
INDEXER_BEHIND = "INDEXER_BEHIND"
INVALID_ADDRESS = "INVALID_ADDRESS"
INVALID_TRANSACTION = "INVALID_TRANSACTION"
MARKET_NOT_FOUND = "MARKET_NOT_FOUND"
PRICE_UNAVAILABLE = "PRICE_UNAVAILABLE"
PERPS_DISABLED = "PERPS_DISABLED"
INVALID_LEVERAGE = "INVALID_LEVERAGE"
INSUFFICIENT_MARGIN = "INSUFFICIENT_MARGIN"
ORDER_REJECTED = "ORDER_REJECTED"
STALE_PRICE = "STALE_PRICE"
SERVICE_UNAVAILABLE = "SERVICE_UNAVAILABLE"
TAP_MINING_DISABLED = "TAP_MINING_DISABLED"
INVALID_TELEGRAM_AUTH = "INVALID_TELEGRAM_AUTH"
ALREADY_CHECKED_IN = "ALREADY_CHECKED_IN"
INSUFFICIENT_POINTS = "INSUFFICIENT_POINTS"
TASK_ALREADY_CLAIMED = "TASK_ALREADY_CLAIMED"
CLAIM_COOLDOWN_ACTIVE = "CLAIM_COOLDOWN_ACTIVE"
NO_PAYOUT_ADDRESS = "NO_PAYOUT_ADDRESS"
FAUCET_DISPATCH_FAILED = "FAUCET_DISPATCH_FAILED"



async def sprax_exception_handler(request: Request, exc: SpraxError) -> JSONResponse:
    request_id = str(uuid.uuid4())
    return JSONResponse(
        status_code=exc.status_code,
        content={
            "success": False,
            "data": None,
            "error": {"code": exc.code, "message": exc.message, "detail": exc.detail},
            "request_id": request_id,
        },
    )


async def http_exception_handler(request: Request, exc: HTTPException) -> JSONResponse:
    request_id = str(uuid.uuid4())
    return JSONResponse(
        status_code=exc.status_code,
        content={
            "success": False,
            "data": None,
            "error": {"code": str(exc.status_code), "message": exc.detail},
            "request_id": request_id,
        },
    )


async def unhandled_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    request_id = str(uuid.uuid4())
    return JSONResponse(
        status_code=500,
        content={
            "success": False,
            "data": None,
            "error": {"code": SERVICE_UNAVAILABLE, "message": "Internal server error"},
            "request_id": request_id,
        },
    )
