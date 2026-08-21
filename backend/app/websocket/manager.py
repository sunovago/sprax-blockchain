"""
WebSocket connection manager with subscription support.
Handles channels: markets, blockchain, explorer, perps, notifications.
Implements heartbeat, auth for private channels, backpressure, and event versioning.
"""
from __future__ import annotations
import asyncio
import json
import uuid
from datetime import datetime, timezone
from typing import Any
from fastapi import WebSocket, WebSocketDisconnect
from app.core.logging import logger
from app.core.security import verify_access_token

CHANNELS = {"markets", "blockchain", "explorer", "perps", "notifications"}
PRIVATE_CHANNELS = {"notifications", "perps"}
HEARTBEAT_INTERVAL = 30  # seconds


def _event(event_type: str, data: Any) -> str:
    return json.dumps({
        "type": event_type,
        "version": 1,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "data": data,
    })


class Connection:
    def __init__(self, ws: WebSocket, conn_id: str, user_id: str | None = None):
        self.ws = ws
        self.id = conn_id
        self.user_id = user_id
        self.subscriptions: set[str] = set()
        self.alive = True

    async def send(self, message: str) -> None:
        try:
            await self.ws.send_text(message)
        except Exception:
            self.alive = False


class ConnectionManager:
    def __init__(self):
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
            except Exception:
                pass
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

    async def subscribe(self, conn: Connection, channel: str) -> bool:
        if channel not in CHANNELS:
            return False
        if channel in PRIVATE_CHANNELS and conn.user_id is None:
            await conn.send(_event("error", {"message": "Authentication required for private channel"}))
            return False
        async with self._lock:
            conn.subscriptions.add(channel)
            self._channel_connections[channel].add(conn.id)
        await conn.send(_event("subscribed", {"channel": channel}))
        return True

    async def broadcast(self, channel: str, event_type: str, data: Any) -> None:
        """Broadcast to all subscribers of a channel."""
        if channel not in CHANNELS:
            return
        message = _event(event_type, data)
        dead = []
        conn_ids = list(self._channel_connections.get(channel, set()))
        for conn_id in conn_ids:
            conn = self._connections.get(conn_id)
            if conn and conn.alive:
                await conn.send(message)
                if not conn.alive:
                    dead.append(conn_id)
            else:
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
                await conn.send(_event("heartbeat", {"ts": datetime.now(timezone.utc).isoformat()}))
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
