import pytest
from unittest.mock import AsyncMock, patch
from httpx import AsyncClient


@pytest.mark.asyncio
async def test_invalid_address(client: AsyncClient):
    response = await client.get("/api/v1/blockchain/accounts/invalid_address")
    assert response.status_code == 400
    data = response.json()
    assert data["success"] is False
    assert data["error"]["code"] == "INVALID_ADDRESS"


@pytest.mark.asyncio
async def test_broadcast_requires_body(client: AsyncClient):
    response = await client.post("/api/v1/blockchain/transactions/broadcast", json={})
    assert response.status_code == 422
