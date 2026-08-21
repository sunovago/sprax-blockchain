import pytest
from httpx import AsyncClient


@pytest.mark.asyncio
async def test_search_empty_results(client: AsyncClient):
    response = await client.get("/api/v1/search?q=nonexistent99999")
    assert response.status_code == 200
    data = response.json()
    assert data["success"] is True
    assert "results" in data["data"]


@pytest.mark.asyncio
async def test_search_missing_q(client: AsyncClient):
    response = await client.get("/api/v1/search")
    assert response.status_code == 422


@pytest.mark.asyncio
async def test_search_valid_address(client: AsyncClient):
    addr = "sprax1" + "b" * 38
    response = await client.get(f"/api/v1/search?q={addr}")
    assert response.status_code == 200
    data = response.json()
    results = data["data"]["results"]
    addr_results = [r for r in results if r["type"] == "address"]
    assert len(addr_results) > 0
