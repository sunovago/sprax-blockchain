import pytest
from httpx import AsyncClient


@pytest.mark.asyncio
async def test_perps_markets_readable(client: AsyncClient):
    response = await client.get("/api/v1/perps/markets")
    assert response.status_code == 200


@pytest.mark.asyncio
async def test_perps_disabled_by_default():
    from app.core.feature_flags import is_enabled
    assert is_enabled("perps_enabled") is False


@pytest.mark.asyncio
async def test_mainnet_disabled_by_default():
    from app.core.feature_flags import is_enabled
    assert is_enabled("mainnet_enabled") is False
