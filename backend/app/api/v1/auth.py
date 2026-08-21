from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from datetime import datetime, timedelta, timezone
from app.core.dependencies import get_db, get_current_user
from app.core.security import create_access_token, create_refresh_token, generate_challenge_nonce
from app.db.models.user import User, Session
from app.db.redis import redis_set, redis_get, redis_delete
from app.core.response import ApiResponse
from app.core.exceptions import SpraxError, UNAUTHORIZED, INVALID_REQUEST
from app.modules.blockchain.address import is_valid_address
import uuid, hashlib

router = APIRouter()


class ChallengeRequest(BaseModel):
    address: str


class VerifyRequest(BaseModel):
    address: str
    nonce: str
    signature: str
    public_key: str


class RefreshRequest(BaseModel):
    refresh_token: str


@router.post("/challenge")
async def request_challenge(req: ChallengeRequest):
    """Issue a replay-resistant challenge nonce for wallet signature auth."""
    if not is_valid_address(req.address):
        raise SpraxError(INVALID_REQUEST, "Invalid Sprax address", 400)
    nonce = generate_challenge_nonce()
    # Store nonce with 5 minute TTL
    await redis_set(f"auth:challenge:{req.address}", nonce, 300)
    return ApiResponse.ok({"address": req.address, "nonce": nonce, "expires_in": 300})


@router.post("/verify")
async def verify_signature(req: VerifyRequest, db: AsyncSession = Depends(get_db)):
    """
    Verifies wallet ownership via signed challenge nonce.
    Backend NEVER receives the private key.
    """
    if not is_valid_address(req.address):
        raise SpraxError(INVALID_REQUEST, "Invalid Sprax address", 400)

    # Check nonce
    stored_nonce = await redis_get(f"auth:challenge:{req.address}")
    if not stored_nonce or stored_nonce != req.nonce:
        raise SpraxError(UNAUTHORIZED, "Invalid or expired challenge nonce", 401)

    # Delete nonce (single use)
    await redis_delete(f"auth:challenge:{req.address}")

    # Cryptographic Ed25519 signature verification
    try:
        pk_str = req.public_key.strip()
        if pk_str.startswith("0x"):
            pk_str = pk_str[2:]
        try:
            pk_bytes = bytes.fromhex(pk_str)
        except ValueError:
            import base64
            pk_bytes = base64.b64decode(pk_str)

        sig_str = req.signature.strip()
        if sig_str.startswith("0x"):
            sig_str = sig_str[2:]
        try:
            sig_bytes = bytes.fromhex(sig_str)
        except ValueError:
            import base64
            sig_bytes = base64.b64decode(sig_str)

        from cryptography.hazmat.primitives.asymmetric import ed25519
        from cryptography.exceptions import InvalidSignature

        if len(pk_bytes) != 32 or len(sig_bytes) != 64:
            raise SpraxError(UNAUTHORIZED, "Invalid public key or signature byte length", 401)

        pub_key = ed25519.Ed25519PublicKey.from_public_bytes(pk_bytes)
        pub_key.verify(sig_bytes, req.nonce.encode("utf-8"))
    except (InvalidSignature, ValueError) as e:
        raise SpraxError(UNAUTHORIZED, f"Signature verification failed: {e}", 401)
    except Exception as e:
        if isinstance(e, SpraxError):
            raise
        raise SpraxError(UNAUTHORIZED, f"Cryptographic verification error: {e}", 401)

    # Find or create user
    result = await db.execute(select(User).where(User.address == req.address))
    user = result.scalar_one_or_none()
    if not user:
        user = User(address=req.address)
        db.add(user)
        await db.flush()

    access_token = create_access_token(str(user.id), extra={"address": user.address})
    refresh_token = create_refresh_token(str(user.id))

    # Store session
    session = Session(
        user_id=user.id,
        refresh_token_hash=hashlib.sha256(refresh_token.encode()).hexdigest(),
        expires_at=datetime.now(timezone.utc) + timedelta(days=30),
    )
    db.add(session)
    await db.commit()

    return ApiResponse.ok({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "token_type": "bearer",
        "address": user.address,
    })


@router.post("/refresh")
async def refresh_token(req: RefreshRequest, db: AsyncSession = Depends(get_db)):
    token_hash = hashlib.sha256(req.refresh_token.encode()).hexdigest()
    result = await db.execute(
        select(Session).where(
            Session.refresh_token_hash == token_hash,
            Session.is_active == True,
            Session.expires_at > datetime.now(timezone.utc),
        )
    )
    session = result.scalar_one_or_none()
    if not session:
        raise SpraxError(UNAUTHORIZED, "Invalid or expired refresh token", 401)

    user_result = await db.execute(select(User).where(User.id == session.user_id))
    user = user_result.scalar_one_or_none()
    if not user:
        raise SpraxError(UNAUTHORIZED, "User not found", 401)

    new_access = create_access_token(str(user.id), extra={"address": user.address})
    return ApiResponse.ok({"access_token": new_access, "token_type": "bearer"})


@router.post("/logout")
async def logout(user: dict = Depends(get_current_user), db: AsyncSession = Depends(get_db)):
    user_id = uuid.UUID(user["sub"])
    result = await db.execute(select(Session).where(Session.user_id == user_id, Session.is_active == True))
    sessions = result.scalars().all()
    for s in sessions:
        s.is_active = False
    await db.commit()
    return ApiResponse.ok({"logged_out": True})
