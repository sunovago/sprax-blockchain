from app.core.config import settings


FLAGS: dict[str, bool] = {
    "perps_enabled": settings.feature_perps_enabled,
    "mainnet_enabled": settings.feature_mainnet_enabled,
    "price_alerts_enabled": settings.feature_price_alerts_enabled,
    "staking_enabled": settings.feature_staking_enabled,
    "markets_enabled": settings.feature_markets_enabled,
    "discover_enabled": settings.feature_discover_enabled,
}


def is_enabled(flag: str) -> bool:
    return FLAGS.get(flag, False)


def get_public_flags() -> dict[str, bool]:
    """Returns only safe public flags for Flutter consumption."""
    return {
        "perps_enabled": FLAGS["perps_enabled"],
        "staking_enabled": FLAGS["staking_enabled"],
        "markets_enabled": FLAGS["markets_enabled"],
        "discover_enabled": FLAGS["discover_enabled"],
        "price_alerts_enabled": FLAGS["price_alerts_enabled"],
    }
