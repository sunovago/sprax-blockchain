from __future__ import annotations
from typing import Any
from app.modules.markets.provider import get_market_provider
from app.db.redis import redis_get, redis_set, TTL_MARKETS, TTL_TICKER, TTL_CANDLES_1H, TTL_TRENDING
from app.core.logging import logger


class MarketService:
    """Market data service with Redis caching."""

    async def get_markets(self) -> list[dict]:
        cached = await redis_get("markets:all")
        if cached:
            return cached
        provider = get_market_provider()
        data = await provider.get_markets()
        normalized = [self._normalize_market(m) for m in data]
        if normalized:
            await redis_set("markets:all", normalized, TTL_MARKETS)
        return normalized

    async def get_ticker(self, symbol: str) -> dict | None:
        key = f"ticker:{symbol.upper()}"
        cached = await redis_get(key)
        if cached:
            return cached
        provider = get_market_provider()
        data = await provider.get_ticker(symbol)
        if data:
            normalized = self._normalize_ticker(data, symbol)
            await redis_set(key, normalized, TTL_TICKER)
            return normalized
        return None

    async def get_candles(self, symbol: str, interval: str = "1h", limit: int = 100) -> list[dict]:
        key = f"candles:{symbol.upper()}:{interval}"
        cached = await redis_get(key)
        if cached:
            return cached
        provider = get_market_provider()
        data = await provider.get_candles(symbol, interval, limit)
        if data:
            await redis_set(key, data, TTL_CANDLES_1H)
        return data

    async def get_gainers(self, limit: int = 10) -> list[dict]:
        markets = await self.get_markets()
        sorted_m = sorted(markets, key=lambda m: m.get("price_change_pct_24h", 0), reverse=True)
        return sorted_m[:limit]

    async def get_losers(self, limit: int = 10) -> list[dict]:
        markets = await self.get_markets()
        sorted_m = sorted(markets, key=lambda m: m.get("price_change_pct_24h", 0))
        return sorted_m[:limit]

    async def get_trending(self) -> list[dict]:
        cached = await redis_get("markets:trending")
        if cached:
            return cached
        markets = await self.get_markets()
        # Trending = highest volume change
        trending = sorted(markets, key=lambda m: m.get("volume_24h", 0), reverse=True)[:10]
        await redis_set("markets:trending", trending, TTL_TRENDING)
        return trending

    async def get_volume_leaders(self, limit: int = 10) -> list[dict]:
        markets = await self.get_markets()
        return sorted(markets, key=lambda m: m.get("volume_24h", 0), reverse=True)[:limit]

    def _normalize_market(self, raw: dict) -> dict:
        return {
            "id": raw.get("id", ""),
            "symbol": raw.get("symbol", "").upper(),
            "name": raw.get("name", ""),
            "price_usd": raw.get("current_price"),
            "price_change_24h": raw.get("price_change_24h"),
            "price_change_pct_24h": raw.get("price_change_percentage_24h"),
            "volume_24h": raw.get("total_volume"),
            "market_cap": raw.get("market_cap"),
            "market_cap_rank": raw.get("market_cap_rank"),
            "high_24h": raw.get("high_24h"),
            "low_24h": raw.get("low_24h"),
            "circulating_supply": raw.get("circulating_supply"),
            "total_supply": raw.get("total_supply"),
            "sparkline": raw.get("sparkline_in_7d", {}).get("price"),
            "logo_url": raw.get("image"),
        }

    def _normalize_ticker(self, raw: dict, symbol: str) -> dict:
        md = raw.get("market_data", {})
        return {
            "symbol": symbol.upper(),
            "name": raw.get("name", symbol),
            "price_usd": md.get("current_price", {}).get("usd"),
            "price_change_pct_24h": md.get("price_change_percentage_24h"),
            "volume_24h": md.get("total_volume", {}).get("usd"),
            "market_cap": md.get("market_cap", {}).get("usd"),
            "high_24h": md.get("high_24h", {}).get("usd"),
            "low_24h": md.get("low_24h", {}).get("usd"),
            "circulating_supply": md.get("circulating_supply"),
            "total_supply": md.get("total_supply"),
        }


_market_service: MarketService | None = None


def get_market_service() -> MarketService:
    global _market_service
    if _market_service is None:
        _market_service = MarketService()
    return _market_service
