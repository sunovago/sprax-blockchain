"""
WebSocket connection manager with subscription support.
Handles channels: markets, blockchain, explorer, perps, notifications.
Implements heartbeat, auth for private channels, backpressure, and event versioning.
"""
from __future__ import annotations

import asyncio
import json
import uuid
from datetime import UTC, datetime
from typing import Any

from fastapi import WebSocket

from app.core.logging import logger
from app.core.security import verify_access_token

CHANNELS = {
    "markets",
    "blockchain",
    "explorer",
    "perps",
    "notifications",
    "newHeads",
    "newBlocks",
    "newTransactions",
    "pendingTransactions",
    "transactionReceipts",
    "validatorUpdates",
    "stakingUpdates",
    "syncStatus",
    "networkStatus",
    "contractEvents",
    "contractExecution",
    "contractDeployment",
}
PRIVATE_CHANNELS = {"notifications", "perps"}
HEARTBEAT_INTERVAL = 30  # seconds


def _event(event_type: str, data: Any, subscription_id: str | None = None) -> str:
    payload = {
        "type": event_type,
        "version": 1,
        "timestamp": datetime.now(UTC).isoformat(),
        "data": data,
    }
    if subscription_id:
        payload["subscription"] = subscription_id
    return json.dumps(payload)


class Connection:
    def __init__(self, ws: WebSocket, conn_id: str, user_id: str | None = None):
        self.ws = ws
        self.id = conn_id
        self.user_id = user_id
        self.subscriptions: set[str] = set()
        self.filters: dict[str, dict[str, Any]] = {}
        self.subscription_ids: dict[str, str] = {}  # channel -> sub_id
        self.alive = True

    async def send(self, message: str) -> None:
        try:
            await self.ws.send_text(message)
        except Exception:
            self.alive = False


class ConnectionManager:
    def __init__(self) -> None:
        self._connections: dict[str, Connection] = {}
        self._channel_connections: dict[str, set[str]] = {ch: set() for ch in CHANNELS}
        self._lock = asyncio.Lock()

    async def connect(self, ws: WebSocket, token: str | None = None) -> Connection:
        await ws.accept()
        conn_id = str(uuid.uuid4())
        user_id = None
        if token:
            try:
                payload = verify_access_token(token)
                user_id = payload.get("sub")
            except Exception as e:
                logger.debug("ws_auth_token_invalid", error=str(e))
        conn = Connection(ws, conn_id, user_id)
        async with self._lock:
            self._connections[conn_id] = conn
        logger.info("ws_connected", conn_id=conn_id, user_id=user_id)
        return conn

    async def disconnect(self, conn_id: str) -> None:
        async with self._lock:
            conn = self._connections.pop(conn_id, None)
            if conn:
                for ch in conn.subscriptions:
                    self._channel_connections.get(ch, set()).discard(conn_id)
        logger.info("ws_disconnected", conn_id=conn_id)

    async def subscribe(
        self,
        conn: Connection,
        channel: str,
        filter_params: dict[str, Any] | None = None,
        rpc_id: Any | None = None,
    ) -> str | None:
        if channel not in CHANNELS:
            if rpc_id is not None:
                await conn.send(json.dumps({
                    "jsonrpc": "2.0",
                    "id": rpc_id,
                    "error": {"code": -32602, "message": f"Unknown channel/topic '{channel}'"}
                }))
            return None
        if channel in PRIVATE_CHANNELS and conn.user_id is None:
            if rpc_id is not None:
                await conn.send(json.dumps({
                    "jsonrpc": "2.0",
                    "id": rpc_id,
                    "error": {"code": -32000, "message": "Authentication required for private channel"}
                }))
            else:
                await conn.send(_event("error", {"message": "Authentication required for private channel"}))
            return None

        sub_id = "0x" + uuid.uuid4().hex[:16]
        async with self._lock:
            conn.subscriptions.add(channel)
            conn.subscription_ids[channel] = sub_id
            if filter_params:
                conn.filters[channel] = filter_params
            self._channel_connections[channel].add(conn.id)

        if rpc_id is not None:
            # JSON-RPC 2.0 sprx_subscribe response
            await conn.send(json.dumps({
                "jsonrpc": "2.0",
                "id": rpc_id,
                "result": sub_id
            }))
        else:
            await conn.send(_event("subscribed", {"channel": channel, "subscription": sub_id}))
        return sub_id

    async def unsubscribe(self, conn: Connection, channel_or_sub_id: str, rpc_id: Any | None = None) -> bool:
        channel_to_remove = None
        for ch, sid in list(conn.subscription_ids.items()):
            if sid == channel_or_sub_id or ch == channel_or_sub_id:
                channel_to_remove = ch
                break

        if not channel_to_remove:
            if rpc_id is not None:
                await conn.send(json.dumps({
                    "jsonrpc": "2.0",
                    "id": rpc_id,
                    "result": False
                }))
            return False

        async with self._lock:
            conn.subscriptions.discard(channel_to_remove)
            conn.subscription_ids.pop(channel_to_remove, None)
            conn.filters.pop(channel_to_remove, None)
            self._channel_connections.get(channel_to_remove, set()).discard(conn.id)

        if rpc_id is not None:
            await conn.send(json.dumps({
                "jsonrpc": "2.0",
                "id": rpc_id,
                "result": True
            }))
        else:
            await conn.send(_event("unsubscribed", {"channel": channel_to_remove}))
        return True

    async def broadcast(self, channel: str, event_type: str, data: Any) -> None:
        """Broadcast to all subscribers of a channel with optional filter matching."""
        if channel not in CHANNELS:
            return
        dead = []
        conn_ids = list(self._channel_connections.get(channel, set()))
        for conn_id in conn_ids:
            conn = self._connections.get(conn_id)
            if not conn or not conn.alive:
                dead.append(conn_id)
                continue

            # Check filters if present (e.g. address filter)
            flt = conn.filters.get(channel)
            if flt:
                target_address = flt.get("address")
                if target_address and isinstance(data, dict):
                    data_addr = data.get("address") or data.get("from") or data.get("to")
                    if data_addr and str(data_addr).lower() != str(target_address).lower():
                        continue

            sub_id = conn.subscription_ids.get(channel)
            message = _event(event_type, data, subscription_id=sub_id)
            await conn.send(message)
            if not conn.alive:
                dead.append(conn_id)

        for conn_id in dead:
            await self.disconnect(conn_id)

    async def send_to_user(self, user_id: str, channel: str, event_type: str, data: Any) -> None:
        """Send to a specific authenticated user (private notifications)."""
        message = _event(event_type, data)
        for conn in list(self._connections.values()):
            if conn.user_id == user_id and channel in conn.subscriptions:
                await conn.send(message)

    async def heartbeat_loop(self) -> None:
        """Sends periodic heartbeats and prunes dead connections."""
        while True:
            await asyncio.sleep(HEARTBEAT_INTERVAL)
            dead = []
            for conn in list(self._connections.values()):
                await conn.send(_event("heartbeat", {"ts": datetime.now(UTC).isoformat()}))
                if not conn.alive:
                    dead.append(conn.id)
            for conn_id in dead:
                await self.disconnect(conn_id)

    @property
    def connection_count(self) -> int:
        return len(self._connections)

    @property
    def channel_stats(self) -> dict[str, int]:
        return {ch: len(conns) for ch, conns in self._channel_connections.items()}


# Singleton
_manager: ConnectionManager | None = None


def get_ws_manager() -> ConnectionManager:
    global _manager
    if _manager is None:
        _manager = ConnectionManager()
    return _manager
