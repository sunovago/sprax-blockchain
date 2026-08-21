import pytest
from app.core.response import ApiResponse, PaginatedResponse


def test_api_response_ok():
    resp = ApiResponse.ok({"key": "value"})
    assert resp.success is True
    assert resp.data == {"key": "value"}
    assert resp.error is None
    assert resp.request_id != ""


def test_api_response_fail():
    resp = ApiResponse.fail("INVALID_REQUEST", "Something went wrong")
    assert resp.success is False
    assert resp.error["code"] == "INVALID_REQUEST"


def test_paginated_response():
    resp = PaginatedResponse.ok([1, 2, 3], total=100, limit=3, offset=0)
    assert resp.success is True
    assert resp.total == 100
    assert resp.has_more is True


def test_paginated_response_no_more():
    resp = PaginatedResponse.ok([1, 2], total=2, limit=10, offset=0)
    assert resp.has_more is False
