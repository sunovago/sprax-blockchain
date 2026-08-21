from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import Any
from app.modules.blockchain.client import get_chain_client
from app.modules.blockchain.address import is_valid_address, is_valid_tx_hash, atto_to_sprx
from app.core.response import ApiResponse
from app.core.exceptions import SpraxError, INVALID_ADDRESS, INVALID_TRANSACTION
from app.core.rate_limit import limiter
from fastapi import Request

router = APIRouter()


class BroadcastRequest(BaseModel):
    body: dict[str, Any]
    key_type: str = "Ed25519"
    public_key: str
    signature: str


@router.get("/status")
async def get_status():
    client = get_chain_client()
    status = await client.get_status()
    return ApiResponse.ok(status)


@router.get("/blocks/{height_or_hash}")
async def get_block(height_or_hash: str):
    client = get_chain_client()
    block = await client.get_block(height_or_hash)
    if not block:
        raise HTTPException(404, "Block not found")
    return ApiResponse.ok(block)


@router.get("/transactions/{hash}")
async def get_transaction(hash: str):
    if not is_valid_tx_hash(hash):
        raise SpraxError(INVALID_TRANSACTION, "Invalid transaction hash format", 400)
    client = get_chain_client()
    tx = await client.get_transaction(hash)
    if not tx:
        raise HTTPException(404, "Transaction not found")
    return ApiResponse.ok(tx)


@router.get("/accounts/{address}")
async def get_account(address: str):
    if not is_valid_address(address):
        raise SpraxError(INVALID_ADDRESS, "Invalid Sprax address format", 400)
    client = get_chain_client()
    account = await client.get_account(address)
    account["balance_sprx"] = atto_to_sprx(account.get("balance", "0"))
    return ApiResponse.ok(account)


@router.get("/accounts/{address}/balance")
async def get_balance(address: str):
    if not is_valid_address(address):
        raise SpraxError(INVALID_ADDRESS, "Invalid Sprax address format", 400)
    client = get_chain_client()
    balance_atto = await client.get_balance(address)
    return ApiResponse.ok({
        "address": address,
        "balance_atto": balance_atto,
        "balance_sprx": atto_to_sprx(balance_atto),
    })


@router.get("/validators")
async def get_validators():
    client = get_chain_client()
    validators = await client.get_validators()
    return ApiResponse.ok(validators)


@router.get("/staking/{address}")
async def get_staking(address: str):
    if not is_valid_address(address):
        raise SpraxError(INVALID_ADDRESS, "Invalid Sprax address format", 400)
    client = get_chain_client()
    staking = await client.get_staking(address)
    delegations = await client.get_delegations(address)
    return ApiResponse.ok({"staking": staking, "delegations": delegations})


@router.post("/transactions/broadcast")
async def broadcast_transaction(request: BroadcastRequest):
    """
    Accepts a signed transaction and broadcasts to chain.
    Backend NEVER receives or stores private keys.
    The wallet signs the transaction offline.
    """
    client = get_chain_client()
    signed_tx = {
        "body": request.body,
        "key_type": request.key_type,
        "public_key": request.public_key,
        "signature": request.signature,
    }
    result = await client.broadcast_signed_transaction(signed_tx)
    return ApiResponse.ok(result)
