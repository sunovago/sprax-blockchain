import uuid
from datetime import datetime
from decimal import Decimal
from sqlalchemy import String, Integer, BigInteger, Boolean, DateTime, Numeric, Text, func
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.dialects.postgresql import UUID, JSONB
from app.db.base import Base


class Asset(Base):
    __tablename__ = "assets"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    symbol: Mapped[str] = mapped_column(String(30), unique=True, nullable=False, index=True)
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    coingecko_id: Mapped[str | None] = mapped_column(String(100), nullable=True)
    market_cap_rank: Mapped[int | None] = mapped_column(Integer, nullable=True)
    is_sprax_native: Mapped[bool] = mapped_column(Boolean, default=False)
    logo_url: Mapped[str | None] = mapped_column(String(500), nullable=True)
    website: Mapped[str | None] = mapped_column(String(255), nullable=True)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    extra_data: Mapped[dict | None] = mapped_column("metadata", JSONB, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class MarketSnapshot(Base):
    __tablename__ = "market_snapshots"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    symbol: Mapped[str] = mapped_column(String(30), nullable=False, index=True)
    price_usd: Mapped[Decimal] = mapped_column(Numeric(30, 10), nullable=False)
    price_change_24h: Mapped[Decimal] = mapped_column(Numeric(20, 10), default=0)
    price_change_pct_24h: Mapped[Decimal] = mapped_column(Numeric(10, 4), default=0)
    volume_24h: Mapped[Decimal] = mapped_column(Numeric(30, 2), default=0)
    market_cap: Mapped[Decimal] = mapped_column(Numeric(30, 2), nullable=True)
    high_24h: Mapped[Decimal] = mapped_column(Numeric(30, 10), nullable=True)
    low_24h: Mapped[Decimal] = mapped_column(Numeric(30, 10), nullable=True)
    circulating_supply: Mapped[Decimal] = mapped_column(Numeric(30, 2), nullable=True)
    total_supply: Mapped[Decimal] = mapped_column(Numeric(30, 2), nullable=True)
    sparkline: Mapped[list | None] = mapped_column(JSONB, nullable=True)
    captured_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), index=True)
