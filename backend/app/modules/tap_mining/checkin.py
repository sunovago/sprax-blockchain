"""Pure day-boundary math for the daily check-in streak — no DB/HTTP concerns."""
from datetime import datetime

from app.core.config import settings


def next_streak(streak: int, last_checkin_at: datetime | None, now: datetime) -> int | None:
    """Returns the new streak count, or None if `now` falls on the same UTC
    calendar day as `last_checkin_at` (already checked in today)."""
    if last_checkin_at is None:
        return 1

    last_day = last_checkin_at.date()
    today = now.date()

    if today == last_day:
        return None
    if (today - last_day).days == 1:
        return streak + 1
    return 1


def checkin_reward(streak: int) -> int:
    capped_streak = min(streak, 7)
    return settings.tap_mining_checkin_base_points + settings.tap_mining_checkin_streak_bonus * (capped_streak - 1)
