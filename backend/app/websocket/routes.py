import json
from datetime import UTC

from fastapi import APIRouter, Query, WebSocket, WebSocketDisconnect

from app.core.logging import logger
from app.websocket.manager import get_ws_manager

ws_router = APIRouter()


async def _handle_ws(channel: str, ws: WebSocket, token: str | None):
    manager = get_ws_manager()
    conn = await manager.connect(ws, token=token)
    # Auto-subscribe to the default channel if specified and not universal
    if channel and channel != "universal":
        await manager.subscribe(conn, channel)
    try:
        while True:
            raw = await ws.receive_text()
            try:
                msg = json.loads(raw)
            except Exception as e:
                logger.debug("ws_message_parse_failed", conn_id=conn.id, error=str(e))
                continue

            # Support JSON-RPC 2.0 subscribe / unsubscribe
            method = msg.get("method")
            if method == "sprx_subscribe":
                params = msg.get("params") or []
                topic = params[0] if len(params) > 0 else "newHeads"
                flt = params[1] if len(params) > 1 and isinstance(params[1], dict) else None
                await manager.subscribe(conn, str(topic), filter_params=flt, rpc_id=msg.get("id"))
                continue
            elif method == "sprx_unsubscribe":
                params = msg.get("params") or []
                sub_id = params[0] if len(params) > 0 else ""
                await manager.unsubscribe(conn, str(sub_id), rpc_id=msg.get("id"))
                continue

            # Support classic action: "subscribe" / "unsubscribe"
            action = msg.get("action")
            if action == "subscribe":
                ch = msg.get("channel", channel)
                flt = msg.get("filter")
                await manager.subscribe(conn, ch, filter_params=flt)
            elif action == "unsubscribe":
                ch = msg.get("channel") or msg.get("subscription") or channel
                await manager.unsubscribe(conn, ch)
            elif action == "ping" or method == "ping":
                import json as _json
                from datetime import datetime
                await conn.send(_json.dumps({"type": "pong", "version": 1, "timestamp": datetime.now(UTC).isoformat(), "data": {}}))
    except (WebSocketDisconnect, Exception) as e:
        logger.info("ws_handler_exit", channel=channel, conn_id=conn.id, reason=str(e))
    finally:
        await manager.disconnect(conn.id)


@ws_router.websocket("/ws")
@ws_router.websocket("/ws/v1")
async def ws_universal(ws: WebSocket, token: str | None = Query(default=None)):
    await _handle_ws("universal", ws, token)


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
