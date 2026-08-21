"""
Market provider abstraction.
Supports CoinGecko as primary provider with clean interface for future providers.
"""
from __future__ import annotations
from abc import ABC, abstractmethod
from typing import Any
import httpx
from app.core.config import settings
from app.core.logging import logger


class MarketProvider(ABC):
    """Abstract market data provider."""

    @abstractmethod
    async def get_markets(self) -> list[dict]: ...

    @abstractmethod
    async def get_ticker(self, symbol: str) -> dict | None: ...

    @abstractmethod
    async def get_candles(self, symbol: str, interval: str, limit: int) -> list[dict]: ...

    @abstractmethod
    async def get_asset_metadata(self, symbol: str) -> dict | None: ...


class CoinGeckoProvider(MarketProvider):
    """CoinGecko market data provider."""

    # Maps SPRX ecosystem symbols to CoinGecko IDs
    SYMBOL_TO_ID: dict[str, str] = {
        "BTC": "bitcoin",
        "ETH": "ethereum",
        "SOL": "solana",
        "BNB": "binancecoin",
        "MATIC": "matic-network",
        "ATOM": "cosmos",
        "AVAX": "avalanche-2",
        "DOT": "polkadot",
        "LINK": "chainlink",
        "UNI": "uniswap",
    }

    def __init__(self):
        headers = {"accept": "application/json"}
        if settings.coingecko_api_key:
            headers["x-cg-pro-api-key"] = settings.coingecko_api_key
        self._client = httpx.AsyncClient(
            base_url=settings.coingecko_base_url,
            headers=headers,
            timeout=15.0,
        )

    async def get_markets(self) -> list[dict]:
        ids = ",".join(self.SYMBOL_TO_ID.values())
        try:
            resp = await self._client.get(
                "/coins/markets",
                params={
                    "vs_currency": "usd",
                    "ids": ids,
                    "order": "market_cap_desc",
                    "per_page": 100,
                    "page": 1,
                    "sparkline": True,
                    "price_change_percentage": "24h",
                },
            )
            resp.raise_for_status()
            return resp.json()
        except Exception as e:
            logger.warning("coingecko_markets_error", error=str(e))
            return []

    async def get_ticker(self, symbol: str) -> dict | None:
        cg_id = self.SYMBOL_TO_ID.get(symbol.upper())
        if not cg_id:
            return None
        try:
            resp = await self._client.get(
                f"/coins/{cg_id}",
                params={"localization": False, "tickers": False, "community_data": False, "developer_data": False},
            )
            resp.raise_for_status()
            return resp.json()
        except Exception as e:
            logger.warning("coingecko_ticker_error", symbol=symbol, error=str(e))
            return None

    async def get_candles(self, symbol: str, interval: str, limit: int = 100) -> list[dict]:
        cg_id = self.SYMBOL_TO_ID.get(symbol.upper())
        if not cg_id:
            return []
        # Map interval to days parameter
        days_map = {"1m": 1, "5m": 1, "15m": 1, "1h": 7, "4h": 30, "1d": 90}
        days = days_map.get(interval, 7)
        try:
            resp = await self._client.get(
                f"/coins/{cg_id}/ohlc",
                params={"vs_currency": "usd", "days": days},
            )
            resp.raise_for_status()
            data = resp.json()
            return [
                {"timestamp": c[0], "open": c[1], "high": c[2], "low": c[3], "close": c[4]}
                for c in data
            ][-limit:]
        except Exception as e:
            logger.warning("coingecko_candles_error", symbol=symbol, error=str(e))
            return []

    async def get_asset_metadata(self, symbol: str) -> dict | None:
        return await self.get_ticker(symbol)

    async def close(self) -> None:
        await self._client.aclose()


_provider: MarketProvider | None = None


def get_market_provider() -> MarketProvider:
    global _provider
    if _provider is None:
        _provider = CoinGeckoProvider()
    return _provider
