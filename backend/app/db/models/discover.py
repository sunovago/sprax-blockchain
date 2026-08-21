import uuid
from datetime import datetime
from decimal import Decimal
from sqlalchemy import String, DateTime, Numeric, Integer, Boolean, Text, func
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.dialects.postgresql import UUID
from app.db.base import Base


class EcosystemProject(Base):
    __tablename__ = "ecosystem_projects"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    project_id: Mapped[str] = mapped_column(String(100), unique=True, nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    tag: Mapped[str] = mapped_column(String(50), nullable=True)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    icon_emoji: Mapped[str | None] = mapped_column(String(10), nullable=True)
    category: Mapped[str] = mapped_column(String(50), nullable=False)  # defi, bridge, tools, validators, infra, nft
    website_url: Mapped[str | None] = mapped_column(String(255), nullable=True)
    tvl_usd: Mapped[Decimal | None] = mapped_column(Numeric(30, 2), nullable=True)
    active_users_24h: Mapped[int | None] = mapped_column(Integer, nullable=True)
    is_verified: Mapped[bool] = mapped_column(Boolean, default=False)
    is_featured: Mapped[bool] = mapped_column(Boolean, default=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    sort_order: Mapped[int] = mapped_column(Integer, default=0)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())
