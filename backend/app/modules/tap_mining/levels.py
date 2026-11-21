"""Pure level/badge tier math for tap mining — no DB/HTTP concerns.

Tier thresholds are game-balance constants, not settings — same reasoning
as `upgrades.UPGRADE_TYPES` being a code constant rather than env config.
"""

LEVELS: list[tuple[int, str]] = [
    (0, "Bronze"),
    (1_000, "Silver"),
    (5_000, "Gold"),
    (20_000, "Platinum"),
    (100_000, "Diamond"),
    (500_000, "Legend"),
]


def get_level(points: int) -> dict:
    index = 0
    for i, (threshold, _name) in enumerate(LEVELS):
        if points >= threshold:
            index = i
        else:
            break

    min_points, name = LEVELS[index]
    next_threshold = LEVELS[index + 1][0] if index + 1 < len(LEVELS) else None

    if next_threshold is None:
        progress = 1.0
    else:
        span = next_threshold - min_points
        progress = (points - min_points) / span if span > 0 else 1.0

    return {
        "name": name,
        "index": index,
        "min_points": min_points,
        "next_threshold": next_threshold,
        "progress": round(min(1.0, max(0.0, progress)), 4),
    }
