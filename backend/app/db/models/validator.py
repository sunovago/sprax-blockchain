import uuid
from datetime import datetime
from decimal import Decimal
from sqlalchemy import String, Integer, BigInteger, Boolean, DateTime, Numeric, Text, func
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.dialects.postgresql import UUID
from app.db.base import Base


class Validator(Base):
    __tablename__ = "validators"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    operator_address: Mapped[str] = mapped_column(String(100), unique=True, nullable=False, index=True)
    consensus_pubkey: Mapped[str | None] = mapped_column(Text, nullable=True)
    moniker: Mapped[str] = mapped_column(String(100), nullable=False)
    identity: Mapped[str | None] = mapped_column(String(64), nullable=True)
    website: Mapped[str | None] = mapped_column(String(255), nullable=True)
    details: Mapped[str | None] = mapped_column(Text, nullable=True)
    commission_rate: Mapped[Decimal] = mapped_column(Numeric(6, 4), default=0)
    commission_max_rate: Mapped[Decimal] = mapped_column(Numeric(6, 4), default=0)
    commission_max_change: Mapped[Decimal] = mapped_column(Numeric(6, 4), default=0)
    bonded_tokens: Mapped[Decimal] = mapped_column(Numeric(38, 0), default=0)
    voting_power: Mapped[int] = mapped_column(BigInteger, default=0)
    voting_power_share: Mapped[Decimal] = mapped_column(Numeric(8, 4), default=0)
    status: Mapped[str] = mapped_column(String(20), default="ACTIVE")  # ACTIVE, JAILED, TOMBSTONED, UNBONDING
    jailed: Mapped[bool] = mapped_column(Boolean, default=False)
    uptime_percent: Mapped[Decimal] = mapped_column(Numeric(5, 2), default=100)
    missed_blocks: Mapped[int] = mapped_column(BigInteger, default=0)
    slash_count: Mapped[int] = mapped_column(Integer, default=0)
    delegator_count: Mapped[int] = mapped_column(Integer, default=0)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class Delegation(Base):
    __tablename__ = "delegations"
    __table_args__ = ({"schema": None},)

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    delegator_address: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    validator_address: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    shares: Mapped[Decimal] = mapped_column(Numeric(38, 8), default=0)
    amount: Mapped[Decimal] = mapped_column(Numeric(38, 0), default=0)  # atto-SPRX
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class StakingEvent(Base):
    __tablename__ = "staking_events"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tx_hash: Mapped[str] = mapped_column(String(66), nullable=False, index=True)
    event_type: Mapped[str] = mapped_column(String(30), nullable=False)  # delegate, undelegate, redelegate, claim_rewards
    delegator_address: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    validator_address: Mapped[str] = mapped_column(String(100), nullable=True)
    amount: Mapped[Decimal] = mapped_column(Numeric(38, 0), default=0)
    block_height: Mapped[int] = mapped_column(BigInteger, nullable=False)
    timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, index=True)
