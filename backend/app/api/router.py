from fastapi import APIRouter
from app.api.v1 import (
    health, blockchain, explorer, markets, fx, search,
    discover, portfolio, validators, staking, watchlists,
    auth, notifications, perps, admin, network, config,
)

api_router = APIRouter()

# Public
api_router.include_router(health.router, prefix="/health", tags=["health"])
api_router.include_router(blockchain.router, prefix="/api/v1/blockchain", tags=["blockchain"])
api_router.include_router(explorer.router, prefix="/api/v1/explorer", tags=["explorer"])
api_router.include_router(markets.router, prefix="/api/v1/markets", tags=["markets"])
api_router.include_router(fx.router, prefix="/api/v1/fx", tags=["fx"])
api_router.include_router(search.router, prefix="/api/v1/search", tags=["search"])
api_router.include_router(discover.router, prefix="/api/v1/discover", tags=["discover"])
api_router.include_router(portfolio.router, prefix="/api/v1/portfolio", tags=["portfolio"])
api_router.include_router(validators.router, prefix="/api/v1/validators", tags=["validators"])
api_router.include_router(staking.router, prefix="/api/v1/staking", tags=["staking"])
api_router.include_router(network.router, prefix="/api/v1/network", tags=["network"])
api_router.include_router(config.router, prefix="/api/v1/config", tags=["config"])

# Auth required
api_router.include_router(auth.router, prefix="/api/v1/auth", tags=["auth"])
api_router.include_router(watchlists.router, prefix="/api/v1/watchlists", tags=["watchlists"])
api_router.include_router(notifications.router, prefix="/api/v1/notifications", tags=["notifications"])

# Perps (read: public, write: auth required)
api_router.include_router(perps.router, prefix="/api/v1/perps", tags=["perps"])

# Admin (internal only)
api_router.include_router(admin.router, prefix="/api/v1/admin", tags=["admin"])
