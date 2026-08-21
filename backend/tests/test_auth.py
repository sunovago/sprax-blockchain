import pytest
from httpx import AsyncClient
from unittest.mock import AsyncMock, patch


@pytest.mark.asyncio
async def test_challenge_invalid_address(client: AsyncClient):
    response = await client.post("/api/v1/auth/challenge", json={"address": "invalid"})
    assert response.status_code == 400
    data = response.json()
    assert data["error"]["code"] == "INVALID_REQUEST"


@pytest.mark.asyncio
async def test_unauthorized_access(client: AsyncClient):
    response = await client.get("/api/v1/watchlists")
    assert response.status_code == 401
