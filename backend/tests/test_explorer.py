import pytest
from httpx import AsyncClient


@pytest.mark.asyncio
async def test_list_blocks_empty(client: AsyncClient):
    response = await client.get("/api/v1/explorer/blocks")
    assert response.status_code == 200
    data = response.json()
    assert data["success"] is True
    assert isinstance(data["data"], list)


@pytest.mark.asyncio
async def test_list_transactions_empty(client: AsyncClient):
    response = await client.get("/api/v1/explorer/transactions")
    assert response.status_code == 200


@pytest.mark.asyncio
async def test_get_block_not_found(client: AsyncClient):
    response = await client.get("/api/v1/explorer/blocks/99999")
    assert response.status_code == 404


@pytest.mark.asyncio
async def test_get_address_not_found(client: AsyncClient):
    addr = "sprax1" + "a" * 38
    response = await client.get(f"/api/v1/explorer/addresses/{addr}")
    assert response.status_code == 404


@pytest.mark.asyncio
async def test_list_validators_empty(client: AsyncClient):
    response = await client.get("/api/v1/explorer/validators")
    assert response.status_code == 200
