"""Pure lazy-regen energy math for tap mining — no DB/HTTP concerns.

Energy regenerates by whole points over elapsed wall-clock time. The refill
anchor (`last_refill_at`) only advances by however many whole regen
intervals were actually consumed, so leftover fractional seconds keep
accruing toward the next point instead of being discarded.

`cap` and `regen_seconds` are passed in explicitly rather than read from
global settings, since upgrades (`app.modules.tap_mining.upgrades`) make
both per-miner values.
"""
from datetime import UTC, datetime, timedelta


def _as_aware(dt: datetime) -> datetime:
    """SQLite (used in tests) doesn't preserve tzinfo on DateTime(timezone=True)
    columns the way Postgres does, so a value loaded fresh from the DB can come
    back naive even though it was written as UTC. Treat any naive value as UTC."""
    return dt if dt.tzinfo is not None else dt.replace(tzinfo=UTC)


def compute_refill(
    energy: int,
    last_refill_at: datetime,
    cap: int,
    regen_seconds: int,
    now: datetime | None = None,
) -> tuple[int, datetime]:
    now = now or datetime.now(UTC)
    last_refill_at = _as_aware(last_refill_at)

    if energy >= cap:
        return cap, now

    elapsed = (now - last_refill_at).total_seconds()
    regenerated = int(elapsed // regen_seconds)
    if regenerated <= 0:
        return energy, last_refill_at

    new_energy = min(cap, energy + regenerated)
    new_last_refill_at = last_refill_at + timedelta(seconds=regenerated * regen_seconds)
    return new_energy, new_last_refill_at


def seconds_until_next_point(
    energy: int,
    last_refill_at: datetime,
    cap: int,
    regen_seconds: int,
    now: datetime | None = None,
) -> int:
    now = now or datetime.now(UTC)
    last_refill_at = _as_aware(last_refill_at)
    if energy >= cap:
        return 0

    elapsed = (now - last_refill_at).total_seconds()
    remainder = elapsed % regen_seconds
    return max(0, int(regen_seconds - remainder))
