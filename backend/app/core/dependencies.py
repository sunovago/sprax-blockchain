from typing import AsyncGenerator
from fastapi import Depends, Header, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession
from app.db.session import async_session_factory
from app.core.security import verify_access_token
from app.core.config import settings


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    async with async_session_factory() as session:
        yield session


async def get_current_user_optional(
    authorization: str | None = Header(default=None),
) -> dict | None:
    if not authorization or not authorization.startswith("Bearer "):
        return None
    token = authorization.split(" ", 1)[1]
    try:
        return verify_access_token(token)
    except Exception:
        return None


async def get_current_user(
    user: dict | None = Depends(get_current_user_optional),
) -> dict:
    if user is None:
        raise HTTPException(status_code=401, detail="Authentication required")
    return user


async def require_admin(
    user: dict = Depends(get_current_user),
) -> dict:
    if user.get("role") not in ("super_admin", "admin"):
        raise HTTPException(status_code=403, detail="Admin access required")
    return user
