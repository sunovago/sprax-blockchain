import pytest
from httpx import AsyncClient


@pytest.mark.asyncio
async def test_public_config(client: AsyncClient):
    response = await client.get("/api/v1/config/public")
    assert response.status_code == 200
    data = response.json()
    assert data["success"] is True
    cfg = data["data"]
    assert "feature_flags" in cfg
    # Perps must be disabled
    assert cfg["feature_flags"]["perps_enabled"] is False
    assert cfg["network"]["decimals"] == 18
