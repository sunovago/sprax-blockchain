from fastapi import APIRouter, WebSocket, WebSocketDisconnect, Query
from app.websocket.manager import get_ws_manager
from app.core.logging import logger
import json

ws_router = APIRouter()


async def _handle_ws(channel: str, ws: WebSocket, token: str | None):
    manager = get_ws_manager()
    conn = await manager.connect(ws, token=token)
    # Auto-subscribe to the channel
    await manager.subscribe(conn, channel)
    try:
        while True:
            raw = await ws.receive_text()
            try:
                msg = json.loads(raw)
            except Exception:
                continue
            action = msg.get("action")
            if action == "subscribe":
                ch = msg.get("channel", channel)
                await manager.subscribe(conn, ch)
            elif action == "unsubscribe":
                ch = msg.get("channel", channel)
                conn.subscriptions.discard(ch)
                manager._channel_connections.get(ch, set()).discard(conn.id)
            elif action == "ping":
                import json as _json
                from datetime import datetime, timezone
                await conn.send(_json.dumps({"type": "pong", "version": 1, "timestamp": datetime.now(timezone.utc).isoformat(), "data": {}}))
    except (WebSocketDisconnect, Exception) as e:
        logger.info("ws_handler_exit", channel=channel, conn_id=conn.id, reason=str(e))
    finally:
        await manager.disconnect(conn.id)


@ws_router.websocket("/ws/markets")
async def ws_markets(ws: WebSocket, token: str | None = Query(default=None)):
    await _handle_ws("markets", ws, token)


@ws_router.websocket("/ws/blockchain")
async def ws_blockchain(ws: WebSocket, token: str | None = Query(default=None)):
    await _handle_ws("blockchain", ws, token)


@ws_router.websocket("/ws/explorer")
async def ws_explorer(ws: WebSocket, token: str | None = Query(default=None)):
    await _handle_ws("explorer", ws, token)


@ws_router.websocket("/ws/perps")
async def ws_perps(ws: WebSocket, token: str | None = Query(default=None)):
    await _handle_ws("perps", ws, token)


@ws_router.websocket("/ws/notifications")
async def ws_notifications(ws: WebSocket, token: str | None = Query(default=None)):
    await _handle_ws("notifications", ws, token)
