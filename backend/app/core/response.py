from typing import Any, TypeVar, Generic
from pydantic import BaseModel
import uuid

T = TypeVar("T")


class ApiResponse(BaseModel):
    success: bool = True
    data: Any = None
    error: Any = None
    request_id: str = ""

    @classmethod
    def ok(cls, data: Any) -> "ApiResponse":
        return cls(success=True, data=data, error=None, request_id=str(uuid.uuid4()))

    @classmethod
    def fail(cls, code: str, message: str) -> "ApiResponse":
        return cls(
            success=False,
            data=None,
            error={"code": code, "message": message},
            request_id=str(uuid.uuid4()),
        )


class PaginatedResponse(BaseModel):
    success: bool = True
    data: list[Any] = []
    total: int = 0
    limit: int = 20
    offset: int = 0
    has_more: bool = False
    request_id: str = ""

    @classmethod
    def ok(cls, data: list[Any], total: int, limit: int, offset: int) -> "PaginatedResponse":
        return cls(
            success=True,
            data=data,
            total=total,
            limit=limit,
            offset=offset,
            has_more=(offset + limit) < total,
            request_id=str(uuid.uuid4()),
        )
