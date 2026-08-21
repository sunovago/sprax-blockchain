"""
SpraxChainClient — Single centralized client for all Sprax Chain RPC interactions.
Chain RPC: JSON-RPC 2.0 on port 26657
Never call chain RPC independently from individual feature modules.
"""
from __future__ import annotations
import httpx
from typing import Any
from app.core.config import settings
from app.core.exceptions import SpraxError, RPC_UNAVAILABLE, INVALID_ADDRESS, INVALID_TRANSACTION
from app.core.logging import logger


class SpraxChainClient:
    """Centralized Sprax Chain RPC client."""

    def __init__(self, rpc_url: str | None = None):
        self._rpc_url = rpc_url or settings.sprax_rpc_url
        self._client = httpx.AsyncClient(timeout=10.0)
        self._id = 0

    def _next_id(self) -> int:
        self._id += 1
        return self._id

    async def _rpc_call(self, method: str, params: list[Any] | None = None) -> Any:
        """Execute a JSON-RPC 2.0 call against the Sprax node."""
        payload = {
            "jsonrpc": "2.0",
            "id": self._next_id(),
            "method": method,
            "params": params or [],
        }
        try:
            resp = await self._client.post(
                self._rpc_url,
                json=payload,
                headers={"Content-Type": "application/json"},
            )
            resp.raise_for_status()
            data = resp.json()
            if "error" in data and data["error"] is not None:
                err = data["error"]
                raise SpraxError(
                    INVALID_TRANSACTION,
                    err.get("message", "RPC error"),
                    400,
                )
            return data.get("result")
        except httpx.HTTPStatusError as e:
            logger.error("rpc_http_error", url=self._rpc_url, status=e.response.status_code)
            raise SpraxError(RPC_UNAVAILABLE, f"RPC HTTP error: {e.response.status_code}", 503)
        except httpx.RequestError as e:
            logger.error("rpc_connection_error", url=self._rpc_url, error=str(e))
            raise SpraxError(RPC_UNAVAILABLE, "Cannot connect to Sprax RPC node", 503)

    # ---- Chain status ----
    async def get_status(self) -> dict:
        """Returns chain ID, latest block height, sync status."""
        return await self._rpc_call("sprax_getStatus") or {}

    async def get_chain_id(self) -> str:
        status = await self.get_status()
        return status.get("chainId", settings.sprax_chain_id)

    async def get_latest_block_height(self) -> int:
        status = await self.get_status()
        return int(status.get("latestBlockHeight", 0))

    # ---- Blocks ----
    async def get_block(self, height_or_hash: str | int) -> dict | None:
        """Get a block by height (int) or hash (str)."""
        return await self._rpc_call("sprax_getBlock", [str(height_or_hash)])

    async def get_latest_block(self) -> dict | None:
        height = await self.get_latest_block_height()
        return await self.get_block(height)

    # ---- Transactions ----
    async def get_transaction(self, tx_hash: str) -> dict | None:
        return await self._rpc_call("sprax_getTransaction", [tx_hash])

    # ---- Accounts ----
    async def get_account(self, address: str) -> dict:
        """Returns {balance: str, nonce: int}."""
        result = await self._rpc_call("sprax_getAccount", [address])
        return result or {"balance": "0", "nonce": 0}

    async def get_balance(self, address: str) -> str:
        """Returns balance in atto-SPRX as string."""
        account = await self.get_account(address)
        return str(account.get("balance", "0"))

    async def get_nonce(self, address: str) -> int:
        account = await self.get_account(address)
        return int(account.get("nonce", 0))

    # ---- Fee estimation ----
    async def estimate_fee(self, tx_body: dict) -> dict:
        """
        Estimate gas/fee for a transaction body.
        Returns default fee if RPC doesn't support estimation.
        """
        try:
            result = await self._rpc_call("sprax_estimateFee", [tx_body])
            return result or {
                "fee": settings.sprax_default_fee_atto,
                "gas_limit": settings.sprax_default_gas,
            }
        except SpraxError:
            return {
                "fee": settings.sprax_default_fee_atto,
                "gas_limit": settings.sprax_default_gas,
            }

    # ---- Broadcast ----
    async def broadcast_signed_transaction(self, signed_tx: dict) -> dict:
        """
        Broadcast a signed transaction to the mempool.
        The wallet signs offline — backend NEVER handles private keys.
        signed_tx: {body, key_type, public_key, signature}
        Returns: {txHash: str}
        """
        result = await self._rpc_call("sprax_broadcastTx", [signed_tx])
        if not result:
            raise SpraxError(INVALID_TRANSACTION, "Transaction broadcast returned no result", 400)
        return result

    # ---- Validators ----
    async def get_validators(self) -> list[dict]:
        result = await self._rpc_call("sprax_getValidators")
        return result if isinstance(result, list) else []

    # ---- Staking ----
    async def get_delegations(self, address: str) -> list[dict]:
        try:
            result = await self._rpc_call("sprax_getDelegations", [address])
            return result if isinstance(result, list) else []
        except SpraxError:
            return []

    async def get_staking(self, address: str) -> dict:
        try:
            result = await self._rpc_call("sprax_getStaking", [address])
            return result or {}
        except SpraxError:
            return {}

    async def close(self) -> None:
        await self._client.aclose()


# Singleton
_chain_client: SpraxChainClient | None = None


def get_chain_client() -> SpraxChainClient:
    global _chain_client
    if _chain_client is None:
        _chain_client = SpraxChainClient()
    return _chain_client
