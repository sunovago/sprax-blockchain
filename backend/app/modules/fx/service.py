"""
FX rates service.
Fetches USD-based fiat conversion rates and caches in Redis.
Never conflates fiat rates with blockchain state.
"""
from __future__ import annotations
import httpx
from app.core.config import settings
from app.db.redis import redis_get, redis_set, TTL_FX
from app.core.logging import logger

SUPPORTED_CURRENCIES = ["USD", "INR", "EUR", "GBP", "JPY"]


class FxService:
    async def get_rates(self) -> dict[str, float]:
        cached = await redis_get("fx:rates")
        if cached:
            return cached
        rates = await self._fetch_rates()
        if rates:
            await redis_set("fx:rates", rates, TTL_FX)
        return rates

    async def convert(self, amount_usd: float, currency: str) -> float:
        rates = await self.get_rates()
        rate = rates.get(currency.upper(), 1.0)
        return round(amount_usd * rate, 6)

    async def _fetch_rates(self) -> dict[str, float]:
        """Fetch from exchangerate-api or fallback to hardcoded approximates."""
        if settings.fx_api_key:
            try:
                async with httpx.AsyncClient(timeout=10.0) as client:
                    resp = await client.get(
                        f"{settings.fx_base_url}/{settings.fx_api_key}/latest/USD"
                    )
                    resp.raise_for_status()
                    data = resp.json()
                    all_rates = data.get("conversion_rates", {})
                    return {c: all_rates.get(c, 1.0) for c in SUPPORTED_CURRENCIES}
            except Exception as e:
                logger.warning("fx_fetch_error", error=str(e))
        # Fallback rates — these MUST be updated regularly in production
        # TODO: configure fx_api_key for live production rates
        return {
            "USD": 1.0,
            "INR": 83.5,
            "EUR": 0.92,
            "GBP": 0.79,
            "JPY": 155.0,
        }


_fx_service: FxService | None = None


def get_fx_service() -> FxService:
    global _fx_service
    if _fx_service is None:
        _fx_service = FxService()
    return _fx_service
