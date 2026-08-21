from fastapi import APIRouter
from app.modules.blockchain.client import get_chain_client
from app.modules.blockchain.address import is_valid_address, atto_to_sprx
from app.core.response import ApiResponse
from app.core.exceptions import SpraxError, INVALID_ADDRESS
from app.core.feature_flags import is_enabled

router = APIRouter()


def _check_staking():
    if not is_enabled("staking_enabled"):
        raise SpraxError("STAKING_DISABLED", "Staking feature is disabled", 503)


@router.get("/validators")
async def get_staking_validators():
    _check_staking()
    client = get_chain_client()
    validators = await client.get_validators()
    return ApiResponse.ok(validators)


@router.get("/delegations/{address}")
async def get_delegations(address: str):
    _check_staking()
    if not is_valid_address(address):
        raise SpraxError(INVALID_ADDRESS, "Invalid Sprax address", 400)
    client = get_chain_client()
    delegations = await client.get_delegations(address)
    return ApiResponse.ok(delegations)


@router.get("/rewards/{address}")
async def get_rewards(address: str):
    _check_staking()
    if not is_valid_address(address):
        raise SpraxError(INVALID_ADDRESS, "Invalid Sprax address", 400)
    client = get_chain_client()
    staking = await client.get_staking(address)
    return ApiResponse.ok({"address": address, "rewards": staking.get("rewards", "0")})


@router.get("/{address}")
async def get_staking_info(address: str):
    _check_staking()
    if not is_valid_address(address):
        raise SpraxError(INVALID_ADDRESS, "Invalid Sprax address", 400)
    client = get_chain_client()
    staking = await client.get_staking(address)
    delegations = await client.get_delegations(address)
    return ApiResponse.ok({
        "address": address,
        "staked_atto": str(staking.get("staked", "0")),
        "staked_sprx": atto_to_sprx(str(staking.get("staked", "0"))),
        "rewards_atto": str(staking.get("rewards", "0")),
        "rewards_sprx": atto_to_sprx(str(staking.get("rewards", "0"))),
        "delegations": delegations,
    })
