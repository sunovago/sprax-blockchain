import uuid
from datetime import datetime, timedelta, timezone
from typing import Any
from jose import jwt, JWTError
from passlib.context import CryptContext
from app.core.config import settings
from app.core.exceptions import SpraxError, UNAUTHORIZED

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")


def hash_password(password: str) -> str:
    return pwd_context.hash(password)


def verify_password(plain: str, hashed: str) -> bool:
    return pwd_context.verify(plain, hashed)


def create_access_token(subject: str, extra: dict[str, Any] | None = None) -> str:
    expire = datetime.now(timezone.utc) + timedelta(
        minutes=settings.jwt_access_token_expire_minutes
    )
    data = {"sub": subject, "exp": expire, "jti": str(uuid.uuid4()), "type": "access"}
    if extra:
        data.update(extra)
    return jwt.encode(data, settings.secret_key, algorithm=settings.jwt_algorithm)


def create_refresh_token(subject: str) -> str:
    expire = datetime.now(timezone.utc) + timedelta(
        days=settings.jwt_refresh_token_expire_days
    )
    data = {"sub": subject, "exp": expire, "jti": str(uuid.uuid4()), "type": "refresh"}
    return jwt.encode(data, settings.secret_key, algorithm=settings.jwt_algorithm)


def verify_access_token(token: str) -> dict:
    try:
        payload = jwt.decode(token, settings.secret_key, algorithms=[settings.jwt_algorithm])
        if payload.get("type") != "access":
            raise SpraxError(UNAUTHORIZED, "Invalid token type", 401)
        return payload
    except JWTError as e:
        raise SpraxError(UNAUTHORIZED, f"Invalid token: {e}", 401)


def generate_challenge_nonce() -> str:
    return str(uuid.uuid4())
