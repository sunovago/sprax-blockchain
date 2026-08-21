import uuid
from datetime import datetime
from decimal import Decimal
from sqlalchemy import String, Integer, BigInteger, Boolean, DateTime, Numeric, Text, Index, func
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.dialects.postgresql import UUID, JSONB
from app.db.base import Base


class Block(Base):
    __tablename__ = "blocks"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    height: Mapped[int] = mapped_column(BigInteger, unique=True, index=True, nullable=False)
    hash: Mapped[str] = mapped_column(String(66), unique=True, index=True, nullable=False)
    parent_hash: Mapped[str | None] = mapped_column(String(66), nullable=True)
    proposer: Mapped[str | None] = mapped_column(String(100), nullable=True)
    state_root: Mapped[str | None] = mapped_column(String(66), nullable=True)
    tx_count: Mapped[int] = mapped_column(Integer, default=0)
    gas_used: Mapped[int] = mapped_column(BigInteger, default=0)
    timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, index=True)
    indexed_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())


class Transaction(Base):
    __tablename__ = "transactions"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    hash: Mapped[str] = mapped_column(String(66), unique=True, index=True, nullable=False)
    block_height: Mapped[int] = mapped_column(BigInteger, nullable=False, index=True)
    block_hash: Mapped[str | None] = mapped_column(String(66), nullable=True)
    sender: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    recipient: Mapped[str | None] = mapped_column(String(100), nullable=True, index=True)
    amount: Mapped[Decimal] = mapped_column(Numeric(38, 0), default=0)  # atto-SPRX, no floats
    fee: Mapped[Decimal] = mapped_column(Numeric(38, 0), default=0)
    gas_limit: Mapped[int] = mapped_column(BigInteger, default=0)
    gas_used: Mapped[int] = mapped_column(BigInteger, default=0)
    nonce: Mapped[int] = mapped_column(BigInteger, default=0)
    memo: Mapped[str | None] = mapped_column(Text, nullable=True)
    tx_type: Mapped[str] = mapped_column(String(50), default="transfer")  # transfer, stake, unstake, etc.
    status: Mapped[str] = mapped_column(String(20), default="success")  # success, failed
    raw: Mapped[dict | None] = mapped_column(JSONB, nullable=True)
    timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, index=True)
    indexed_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())


class Address(Base):
    __tablename__ = "addresses"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    address: Mapped[str] = mapped_column(String(100), unique=True, nullable=False, index=True)
    balance: Mapped[Decimal] = mapped_column(Numeric(38, 0), default=0)  # atto-SPRX
    nonce: Mapped[int] = mapped_column(BigInteger, default=0)
    tx_count: Mapped[int] = mapped_column(Integer, default=0)
    first_seen_height: Mapped[int] = mapped_column(BigInteger, nullable=True)
    last_seen_height: Mapped[int] = mapped_column(BigInteger, nullable=True)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class Transfer(Base):
    __tablename__ = "transfers"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tx_hash: Mapped[str] = mapped_column(String(66), nullable=False, index=True)
    block_height: Mapped[int] = mapped_column(BigInteger, nullable=False)
    from_address: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    to_address: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    amount: Mapped[Decimal] = mapped_column(Numeric(38, 0), nullable=False)
    timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, index=True)
