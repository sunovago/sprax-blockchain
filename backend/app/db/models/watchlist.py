import uuid
from datetime import datetime
from sqlalchemy import String, ForeignKey, DateTime, UniqueConstraint, func
from sqlalchemy.orm import Mapped, mapped_column, relationship
from sqlalchemy.dialects.postgresql import UUID
from app.db.base import Base


class Watchlist(Base):
    __tablename__ = "watchlists"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    user_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("users.id", ondelete="CASCADE"), index=True)
    name: Mapped[str] = mapped_column(String(100), default="My Watchlist")
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    user: Mapped["User"] = relationship(back_populates="watchlists")
    assets: Mapped[list["WatchlistAsset"]] = relationship(back_populates="watchlist", cascade="all, delete-orphan")


class WatchlistAsset(Base):
    __tablename__ = "watchlist_assets"
    __table_args__ = (UniqueConstraint("watchlist_id", "symbol", name="uq_watchlist_asset"),)

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    watchlist_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("watchlists.id", ondelete="CASCADE"), index=True)
    symbol: Mapped[str] = mapped_column(String(30), nullable=False)
    added_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())

    watchlist: Mapped["Watchlist"] = relationship(back_populates="assets")
