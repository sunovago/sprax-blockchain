import uuid
from datetime import datetime
from decimal import Decimal
from sqlalchemy import String, ForeignKey, DateTime, Numeric, BigInteger, Boolean, func
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.dialects.postgresql import UUID, JSONB
from app.db.base import Base


class PerpMarket(Base):
    __tablename__ = "perp_markets"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    symbol: Mapped[str] = mapped_column(String(30), unique=True, nullable=False, index=True)
    base_asset: Mapped[str] = mapped_column(String(20), nullable=False)
    quote_asset: Mapped[str] = mapped_column(String(20), nullable=False)
    max_leverage: Mapped[int] = mapped_column(BigInteger, default=50)
    min_leverage: Mapped[int] = mapped_column(BigInteger, default=1)
    maker_fee: Mapped[Decimal] = mapped_column(Numeric(6, 4), default=Decimal("0.0002"))
    taker_fee: Mapped[Decimal] = mapped_column(Numeric(6, 4), default=Decimal("0.0006"))
    mark_price: Mapped[Decimal] = mapped_column(Numeric(30, 10), default=0)
    index_price: Mapped[Decimal] = mapped_column(Numeric(30, 10), default=0)
    last_price: Mapped[Decimal] = mapped_column(Numeric(30, 10), default=0)
    funding_rate: Mapped[Decimal] = mapped_column(Numeric(12, 8), default=0)
    open_interest: Mapped[Decimal] = mapped_column(Numeric(30, 8), default=0)
    is_active: Mapped[bool] = mapped_column(Boolean, default=False)  # disabled until mainnet perps audit
    is_testnet_only: Mapped[bool] = mapped_column(Boolean, default=True)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class Order(Base):
    __tablename__ = "orders"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("users.id", ondelete="CASCADE"), index=True)
    symbol: Mapped[str] = mapped_column(String(30), nullable=False, index=True)
    side: Mapped[str] = mapped_column(String(10), nullable=False)  # long, short
    order_type: Mapped[str] = mapped_column(String(20), nullable=False)  # market, limit, stop_limit
    size: Mapped[Decimal] = mapped_column(Numeric(30, 8), nullable=False)
    price: Mapped[Decimal | None] = mapped_column(Numeric(30, 10), nullable=True)
    leverage: Mapped[int] = mapped_column(BigInteger, default=1)
    margin: Mapped[Decimal] = mapped_column(Numeric(30, 8), default=0)
    status: Mapped[str] = mapped_column(String(20), default="open")  # open, filled, cancelled, rejected
    is_testnet: Mapped[bool] = mapped_column(Boolean, default=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class Position(Base):
    __tablename__ = "positions"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("users.id", ondelete="CASCADE"), index=True)
    symbol: Mapped[str] = mapped_column(String(30), nullable=False, index=True)
    side: Mapped[str] = mapped_column(String(10), nullable=False)
    size: Mapped[Decimal] = mapped_column(Numeric(30, 8), nullable=False)
    entry_price: Mapped[Decimal] = mapped_column(Numeric(30, 10), nullable=False)
    mark_price: Mapped[Decimal] = mapped_column(Numeric(30, 10), default=0)
    leverage: Mapped[int] = mapped_column(BigInteger, default=1)
    margin: Mapped[Decimal] = mapped_column(Numeric(30, 8), nullable=False)
    liquidation_price: Mapped[Decimal] = mapped_column(Numeric(30, 10), nullable=True)
    unrealized_pnl: Mapped[Decimal] = mapped_column(Numeric(20, 8), default=0)
    is_open: Mapped[bool] = mapped_column(Boolean, default=True)
    is_testnet: Mapped[bool] = mapped_column(Boolean, default=True)
    opened_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class Trade(Base):
    __tablename__ = "trades"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    symbol: Mapped[str] = mapped_column(String(30), nullable=False, index=True)
    price: Mapped[Decimal] = mapped_column(Numeric(30, 10), nullable=False)
    size: Mapped[Decimal] = mapped_column(Numeric(30, 8), nullable=False)
    side: Mapped[str] = mapped_column(String(10), nullable=False)
    is_testnet: Mapped[bool] = mapped_column(Boolean, default=True)
    timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), index=True)


class FundingEvent(Base):
    __tablename__ = "funding_events"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    symbol: Mapped[str] = mapped_column(String(30), nullable=False, index=True)
    rate: Mapped[Decimal] = mapped_column(Numeric(12, 8), nullable=False)
    mark_price: Mapped[Decimal] = mapped_column(Numeric(30, 10), nullable=False)
    timestamp: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), index=True)
