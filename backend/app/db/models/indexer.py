import uuid
from datetime import datetime
from decimal import Decimal
from sqlalchemy import String, BigInteger, DateTime, Numeric, Boolean, func
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.dialects.postgresql import UUID
from app.db.base import Base


class IndexerState(Base):
    __tablename__ = "indexer_state"

    id: Mapped[int] = mapped_column(primary_key=True, default=1)  # singleton row
    latest_chain_height: Mapped[int] = mapped_column(BigInteger, default=0)
    latest_indexed_height: Mapped[int] = mapped_column(BigInteger, default=0)
    lag_blocks: Mapped[int] = mapped_column(BigInteger, default=0)
    is_syncing: Mapped[bool] = mapped_column(Boolean, default=False)
    last_error: Mapped[str | None] = mapped_column(String(500), nullable=True)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())


class NetworkStat(Base):
    __tablename__ = "network_stats"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    block_height: Mapped[int] = mapped_column(BigInteger, nullable=False)
    total_transactions: Mapped[int] = mapped_column(BigInteger, default=0)
    active_validators: Mapped[int] = mapped_column(BigInteger, default=0)
    total_staked: Mapped[Decimal] = mapped_column(Numeric(38, 0), default=0)
    tps: Mapped[Decimal] = mapped_column(Numeric(10, 2), default=0)
    avg_block_time_ms: Mapped[int] = mapped_column(BigInteger, default=2000)
    captured_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), index=True)
