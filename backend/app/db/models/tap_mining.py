import uuid
from datetime import datetime

from sqlalchemy import BigInteger, Boolean, DateTime, ForeignKey, Integer, String, Text, UniqueConstraint, func
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base


class TapMiner(Base):
    """Standalone tap-to-earn identity, keyed by Telegram user id.

    Deliberately NOT linked to `User` (wallet auth) — players don't need
    a wallet to play. Points are off-chain only; no relation to the Rust
    chain or any mint/payout logic exists here.
    """

    __tablename__ = "tap_miners"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    telegram_id: Mapped[int] = mapped_column(BigInteger, unique=True, index=True, nullable=False)
    telegram_username: Mapped[str | None] = mapped_column(String(100), nullable=True)
    telegram_first_name: Mapped[str | None] = mapped_column(String(100), nullable=True)
    telegram_last_name: Mapped[str | None] = mapped_column(String(100), nullable=True)
    telegram_photo_url: Mapped[str | None] = mapped_column(String(500), nullable=True)

    points: Mapped[int] = mapped_column(BigInteger, default=0)
    energy: Mapped[int] = mapped_column(Integer, default=0)
    energy_last_refill_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    taps_total: Mapped[int] = mapped_column(BigInteger, default=0)
    last_tap_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    is_banned: Mapped[bool] = mapped_column(Boolean, default=False)

    referral_code: Mapped[str] = mapped_column(String(16), unique=True, index=True, nullable=False)
    referred_by_id: Mapped[uuid.UUID | None] = mapped_column(
        ForeignKey("tap_miners.id", ondelete="SET NULL"), index=True, nullable=True
    )
    referral_earnings: Mapped[int] = mapped_column(BigInteger, default=0)

    checkin_streak: Mapped[int] = mapped_column(Integer, default=0)
    longest_checkin_streak: Mapped[int] = mapped_column(Integer, default=0)
    last_checkin_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    energy_cap_level: Mapped[int] = mapped_column(Integer, default=0)
    regen_speed_level: Mapped[int] = mapped_column(Integer, default=0)
    multitap_level: Mapped[int] = mapped_column(Integer, default=0)

    payout_address: Mapped[str | None] = mapped_column(String(100), nullable=True)
    last_claimed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    total_sprx_claimed: Mapped[int] = mapped_column(BigInteger, default=0)

    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), onupdate=func.now()
    )


class TapMinerActivity(Base):
    """Discrete game-event log for the Profile tab's activity history.

    Deliberately excludes individual tap batches — at up to one row/second
    per active player that would grow unbounded for no product value; only
    check-ins, referrals, upgrade purchases, and level-ups are logged.
    """

    __tablename__ = "tap_mining_activity"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tap_miner_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("tap_miners.id", ondelete="CASCADE"), index=True, nullable=False
    )
    event_type: Mapped[str] = mapped_column(String(30), nullable=False)
    points_delta: Mapped[int] = mapped_column(BigInteger, default=0)
    detail: Mapped[str | None] = mapped_column(String(200), nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), index=True)


class TapMiningTask(Base):
    """Admin-authored task/quest catalog. One flexible shape covers all three
    task types rather than three separate tables — `verify_target`/
    `verify_value` are simply unused for "manual" tasks.
    """

    __tablename__ = "tap_mining_tasks"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    title: Mapped[str] = mapped_column(String(200), nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    task_type: Mapped[str] = mapped_column(String(20), nullable=False)  # manual | telegram_join | milestone
    reward_points: Mapped[int] = mapped_column(BigInteger, default=0)

    action_url: Mapped[str | None] = mapped_column(String(500), nullable=True)
    verify_target: Mapped[str | None] = mapped_column(String(200), nullable=True)
    verify_value: Mapped[int | None] = mapped_column(Integer, nullable=True)

    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    sort_order: Mapped[int] = mapped_column(Integer, default=0)

    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), onupdate=func.now()
    )


class TapMinerTaskCompletion(Base):
    """A row's mere existence means the task is claimed — prevents double-claim
    via the (tap_miner_id, task_id) unique constraint."""

    __tablename__ = "tap_mining_task_completions"
    __table_args__ = (UniqueConstraint("tap_miner_id", "task_id", name="uq_tap_miner_task"),)

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tap_miner_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("tap_miners.id", ondelete="CASCADE"), index=True, nullable=False
    )
    task_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("tap_mining_tasks.id", ondelete="CASCADE"), index=True, nullable=False
    )
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now())


class TapMinerClaim(Base):
    """Record of an on-chain tSPRX payout claim by a player.
    Tracks points debited, tokens minted/disbursed, recipient address,
    and on-chain transaction hash.
    """

    __tablename__ = "tap_mining_claims"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    tap_miner_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("tap_miners.id", ondelete="CASCADE"), index=True, nullable=False
    )
    points_spent: Mapped[int] = mapped_column(BigInteger, nullable=False)
    sprx_amount: Mapped[int] = mapped_column(BigInteger, nullable=False)
    recipient_address: Mapped[str] = mapped_column(String(100), nullable=False)
    status: Mapped[str] = mapped_column(String(20), default="pending", index=True)  # pending | completed | failed
    tx_hash: Mapped[str | None] = mapped_column(String(66), nullable=True)
    error_message: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), server_default=func.now(), index=True)
    completed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

