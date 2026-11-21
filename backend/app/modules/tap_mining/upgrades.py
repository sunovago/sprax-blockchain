"""Pure math for the tap-mining upgrade shop — no DB/HTTP concerns.

Energy cap, regen speed, and points-per-tap all become per-miner values
derived from these level columns, rather than flat global settings.
"""
from app.core.config import settings

UPGRADE_TYPES = ("energy_cap", "regen_speed", "multitap")


def effective_energy_cap(level: int) -> int:
    return settings.tap_mining_energy_cap + level * settings.tap_mining_upgrade_energy_cap_step


def effective_regen_seconds(level: int) -> int:
    return max(
        settings.tap_mining_upgrade_min_regen_seconds,
        settings.tap_mining_energy_regen_seconds - level * settings.tap_mining_upgrade_regen_step,
    )


def effective_points_per_tap(level: int) -> int:
    return settings.tap_mining_points_per_tap + level * settings.tap_mining_upgrade_multitap_step


def upgrade_cost(current_level: int) -> int:
    return int(settings.tap_mining_upgrade_base_cost * (settings.tap_mining_upgrade_cost_multiplier**current_level))
