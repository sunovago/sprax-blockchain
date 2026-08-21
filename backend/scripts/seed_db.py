"""
Seed script for initial database data.
Run with: python scripts/seed_db.py
"""
import asyncio
from decimal import Decimal
from sqlalchemy.ext.asyncio import AsyncSession
from app.db.session import async_session_factory
from app.db.models.discover import EcosystemProject
from app.db.models.admin import AdminUser, FeatureFlag
from app.db.models.perp import PerpMarket
from app.core.security import hash_password
from app.core.config import settings

ECOSYSTEM_PROJECTS = [
    {"project_id": "sprax_swap", "name": "SpraxSwap DEX", "tag": "DEX",
     "description": "Ultra-low slippage concentrated liquidity AMM native to Sprax Chain.",
     "icon_emoji": "⚡", "category": "defi", "website_url": "https://swap.sprax.network",
     "tvl_usd": Decimal("34500000"), "active_users_24h": 12400, "is_verified": True, "sort_order": 1},
    {"project_id": "sprax_perp", "name": "SpraxPerp Protocol", "tag": "Perpetuals",
     "description": "Decentralized perpetual futures with up to 50x leverage.",
     "icon_emoji": "📈", "category": "defi", "website_url": "https://perps.sprax.network",
     "tvl_usd": Decimal("28400000"), "active_users_24h": 8900, "is_verified": True, "sort_order": 2},
    {"project_id": "sprax_lend", "name": "SpraxLend Money Market", "tag": "Lending",
     "description": "Permissionless lending & borrowing for SPRX, sUSD, and wrapped assets.",
     "icon_emoji": "🏦", "category": "defi", "website_url": "https://lend.sprax.network",
     "tvl_usd": Decimal("19200000"), "active_users_24h": 4500, "is_verified": True, "sort_order": 3},
    {"project_id": "sprax_bridge", "name": "Sprax Teleport Bridge", "tag": "Bridge",
     "description": "Zero-knowledge cross-chain relayer linking Sprax with Ethereum, Solana, and Cosmos.",
     "icon_emoji": "🌉", "category": "bridge", "website_url": "https://bridge.sprax.network",
     "tvl_usd": Decimal("42000000"), "active_users_24h": 6200, "is_verified": True, "sort_order": 4},
    {"project_id": "sprax_explorer", "name": "SpraxScan Block Explorer", "tag": "Explorer",
     "description": "Real-time blockchain explorer for blocks, addresses, and validator metrics.",
     "icon_emoji": "🔍", "category": "tools", "website_url": "https://explorer.sprax.network",
     "is_verified": True, "sort_order": 5},
    {"project_id": "sprax_validator_hub", "name": "Validator Staking Portal", "tag": "Staking",
     "description": "Official staking registry with commission stats and auto-compound tools.",
     "icon_emoji": "🛡️", "category": "validators", "website_url": "https://validators.sprax.network",
     "tvl_usd": Decimal("68400000"), "active_users_24h": 3100, "is_verified": True, "sort_order": 6},
    {"project_id": "sprax_oracle", "name": "SpraxFeed Decentralized Oracles", "tag": "Oracles",
     "description": "High-frequency sub-second price feeds powering perps and lending.",
     "icon_emoji": "📡", "category": "infra", "website_url": "https://oracles.sprax.network",
     "is_verified": True, "sort_order": 7},
    {"project_id": "sprax_domain", "name": "Sprax Name Service (.sprx)", "tag": "Domains",
     "description": "Human-readable decentralized handles for Sprax addresses.",
     "icon_emoji": "🏷️", "category": "tools", "website_url": "https://sns.sprax.network",
     "active_users_24h": 1800, "is_verified": True, "sort_order": 8},
]

PERP_MARKETS = [
    {"symbol": "SPRX/USDT", "base_asset": "SPRX", "quote_asset": "USDT", "max_leverage": 50, "is_active": False, "is_testnet_only": True},
    {"symbol": "BTC/USDT", "base_asset": "BTC", "quote_asset": "USDT", "max_leverage": 100, "is_active": False, "is_testnet_only": True},
    {"symbol": "ETH/USDT", "base_asset": "ETH", "quote_asset": "USDT", "max_leverage": 50, "is_active": False, "is_testnet_only": True},
    {"symbol": "SOL/USDT", "base_asset": "SOL", "quote_asset": "USDT", "max_leverage": 25, "is_active": False, "is_testnet_only": True},
]

FEATURE_FLAGS = [
    {"name": "perps_enabled", "is_enabled": False, "description": "BLOCKED — requires full audit and explicit human approval"},
    {"name": "mainnet_enabled", "is_enabled": False, "description": "BLOCKED — requires genesis ceremony"},
    {"name": "markets_enabled", "is_enabled": True, "description": "Market data display"},
    {"name": "discover_enabled", "is_enabled": True, "description": "Discover ecosystem page"},
    {"name": "staking_enabled", "is_enabled": True, "description": "Staking and validators"},
    {"name": "price_alerts_enabled", "is_enabled": True, "description": "User price alert notifications"},
]


async def seed():
    async with async_session_factory() as db:
        # Ecosystem projects
        for p in ECOSYSTEM_PROJECTS:
            proj = EcosystemProject(**p)
            db.add(proj)

        # Perp markets (all disabled / testnet only)
        for m in PERP_MARKETS:
            market = PerpMarket(**m)
            db.add(market)

        # Feature flags
        for f in FEATURE_FLAGS:
            flag = FeatureFlag(**f)
            db.add(flag)

        # Default admin user (change password immediately in production)
        admin = AdminUser(
            username="admin",
            password_hash=hash_password("change-me-immediately"),
            role="super_admin",
        )
        db.add(admin)

        await db.commit()
        print("Database seeded successfully.")


if __name__ == "__main__":
    asyncio.run(seed())
