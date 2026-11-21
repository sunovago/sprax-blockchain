"""
SpraxChainClient — Single centralized client for all Sprax Chain RPC interactions.
Chain RPC: JSON-RPC 2.0 on port 26657
Never call chain RPC independently from individual feature modules.
"""
from __future__ import annotations

from typing import Any

import httpx

from app.core.config import settings
from app.core.exceptions import INVALID_TRANSACTION, RPC_UNAVAILABLE, SpraxError
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

    async def get_chain_info(self) -> dict:
        """Returns detailed chain metadata, genesis, and consensus params."""
        return await self._rpc_call("sprx_getChainInfo") or {}

    async def get_sync_status(self) -> dict:
        """Returns node synchronization status."""
        return await self._rpc_call("sprx_getSyncStatus") or {}

    # ---- Blocks ----
    async def get_block(self, height_or_hash: str | int) -> dict | None:
        """Get a block by height (int) or hash (str)."""
        return await self._rpc_call("sprax_getBlock", [str(height_or_hash)])

    async def get_block_by_number(self, height: int, full_tx: bool = False) -> dict | None:
        return await self._rpc_call("sprx_getBlockByNumber", [height, full_tx])

    async def get_block_by_hash(self, block_hash: str, full_tx: bool = False) -> dict | None:
        return await self._rpc_call("sprx_getBlockByHash", [block_hash, full_tx])

    async def get_latest_block(self) -> dict | None:
        height = await self.get_latest_block_height()
        return await self.get_block(height)

    # ---- Transactions ----
    async def get_transaction(self, tx_hash: str) -> dict | None:
        return await self._rpc_call("sprax_getTransaction", [tx_hash])

    async def get_transaction_by_hash(self, tx_hash: str) -> dict | None:
        return await self._rpc_call("sprx_getTransactionByHash", [tx_hash])

    async def get_transaction_receipt(self, tx_hash: str) -> dict | None:
        return await self._rpc_call("sprx_getTransactionReceipt", [tx_hash])

    async def get_transaction_status(self, tx_hash: str) -> dict | None:
        return await self._rpc_call("sprx_getTransactionStatus", [tx_hash])

    async def get_pending_transactions(self) -> list[dict]:
        res = await self._rpc_call("sprx_getPendingTransactions")
        return res if isinstance(res, list) else []

    async def send_raw_transaction(self, raw_tx: str | dict) -> dict:
        """Submits signed raw transaction hex or JSON."""
        return await self._rpc_call("sprx_sendRawTransaction", [raw_tx])

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

    async def get_account_state(self, address: str) -> dict:
        return await self._rpc_call("sprx_getAccountState", [address]) or {}

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

    async def get_validator(self, address: str) -> dict | None:
        return await self._rpc_call("sprx_getValidator", [address])

    async def get_validator_status(self, address: str) -> dict | None:
        return await self._rpc_call("sprx_getValidatorStatus", [address])

    async def get_validator_voting_power(self, address: str) -> dict | None:
        return await self._rpc_call("sprx_getValidatorVotingPower", [address])

    async def get_validator_rewards(self, address: str) -> dict | None:
        return await self._rpc_call("sprx_getValidatorRewards", [address])

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

    async def get_staking_info(self) -> dict:
        return await self._rpc_call("sprx_getStakingInfo") or {}

    async def get_unbonding_requests(self, address: str) -> list[dict]:
        res = await self._rpc_call("sprx_getUnbondingRequests", [address])
        return res if isinstance(res, list) else []

    # ---- Network & Node ----
    async def get_network_peers(self) -> list[dict]:
        res = await self._rpc_call("sprx_netPeers")
        return res if isinstance(res, list) else []

    async def get_node_info(self) -> dict:
        return await self._rpc_call("sprx_nodeInfo") or {}

    async def get_node_health(self) -> dict:
        return await self._rpc_call("sprx_nodeHealth") or {}

    # ---- Smart Contracts ----
    async def deploy_contract(self, signed_tx: dict) -> dict:
        """Broadcast a signed contract deployment transaction."""
        return await self._rpc_call("sprx_deployContract", [signed_tx])

    async def call_contract(self, signed_tx: dict) -> dict:
        """Broadcast a signed contract call transaction."""
        return await self._rpc_call("sprx_callContract", [signed_tx])

    async def query_contract(self, address: str, msg: dict | str, gas: int = 200_000) -> dict:
        """Execute a read-only query against a deployed contract."""
        return await self._rpc_call("sprx_queryContract", [address, msg, gas]) or {}

    async def get_contract(self, address: str) -> dict | None:
        """Retrieve contract metadata (code_id, creator, admin, label)."""
        return await self._rpc_call("sprx_getContract", [address])

    async def get_contract_code(self, address_or_code_hash: str) -> dict | None:
        """Retrieve contract bytecode."""
        return await self._rpc_call("sprx_getContractCode", [address_or_code_hash])

    async def get_contract_storage(self, address: str, key: str) -> dict | None:
        """Retrieve raw storage key value."""
        return await self._rpc_call("sprx_getContractStorage", [address, key])

    async def get_contract_events(self, address: str, limit: int = 50) -> list[dict]:
        """Retrieve recent events emitted by a contract."""
        res = await self._rpc_call("sprx_getContractEvents", [address, limit])
        return res if isinstance(res, list) else []

    async def estimate_contract_gas(self, address: str, msg: dict, funds: str | None = None) -> dict:
        """Estimate gas and fee required for a contract interaction."""
        return await self._rpc_call("sprx_estimateContractGas", [address, msg, funds]) or {}

    async def get_contract_receipt(self, tx_hash: str) -> dict | None:
        """Retrieve execution receipt including contract return data and events."""
        return await self._rpc_call("sprx_getContractReceipt", [tx_hash])

    async def close(self) -> None:
        await self._client.aclose()


# Singleton
_chain_client: SpraxChainClient | None = None


def get_chain_client() -> SpraxChainClient:
    global _chain_client
    if _chain_client is None:
        _chain_client = SpraxChainClient()
    return _chain_client
