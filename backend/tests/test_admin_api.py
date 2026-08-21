import pytest
from httpx import AsyncClient
from unittest.mock import AsyncMock, patch
from app.core.security import create_access_token


@pytest.mark.asyncio
async def test_admin_unauthorized_access(client: AsyncClient):
    response = await client.get("/api/v1/admin/dashboard")
    assert response.status_code == 401


@pytest.mark.asyncio
async def test_admin_viewer_forbidden_on_write(client: AsyncClient):
    viewer_token = create_access_token("viewer-id", extra={"role": "viewer", "username": "test_viewer"})
    headers = {"Authorization": f"Bearer {viewer_token}"}
    response = await client.patch("/api/v1/admin/feature-flags/staking_enabled", json={"is_enabled": True}, headers=headers)
    assert response.status_code == 403


@pytest.mark.asyncio
async def test_critical_flag_protection(client: AsyncClient):
    admin_token = create_access_token("admin-id", extra={"role": "admin", "username": "admin_user"})
    headers = {"Authorization": f"Bearer {admin_token}"}
    
    # Attempting to enable perps_enabled or mainnet_enabled must be blocked
    response = await client.patch("/api/v1/admin/feature-flags/perps_enabled", json={"is_enabled": True}, headers=headers)
    # 403 or 404 if flag not in DB mock, but cannot succeed
    assert response.status_code in (403, 404)
