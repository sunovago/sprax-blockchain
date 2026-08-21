from fastapi import APIRouter
from app.modules.blockchain.client import get_chain_client
from app.modules.blockchain.address import is_valid_address, atto_to_sprx
from app.modules.markets.service import get_market_service
from app.modules.fx.service import get_fx_service
from app.core.response import ApiResponse
from app.core.exceptions import SpraxError, INVALID_ADDRESS

router = APIRouter()


@router.get("/{address}")
async def get_portfolio(address: str, currency: str = "USD"):
    if not is_valid_address(address):
        raise SpraxError(INVALID_ADDRESS, "Invalid Sprax address", 400)

    client = get_chain_client()
    market_svc = get_market_service()
    fx_svc = get_fx_service()

    import asyncio
    account, delegations, staking, markets = await asyncio.gather(
        client.get_account(address),
        client.get_delegations(address),
        client.get_staking(address),
        market_svc.get_markets(),
        return_exceptions=True,
    )

    if isinstance(account, Exception):
        account = {"balance": "0", "nonce": 0}
    if isinstance(delegations, Exception):
        delegations = []
    if isinstance(staking, Exception):
        staking = {}
    if isinstance(markets, Exception):
        markets = []

    balance_atto = str(account.get("balance", "0"))
    balance_sprx_str = atto_to_sprx(balance_atto)

    # Get SPRX price
    sprx_market = next((m for m in markets if m.get("symbol", "").upper() == "SPRX"), None)
    sprx_price_usd = float(sprx_market["price_usd"]) if sprx_market and sprx_market.get("price_usd") else 0.0

    try:
        balance_float = float(balance_sprx_str)
    except ValueError:
        balance_float = 0.0

    balance_usd = balance_float * sprx_price_usd
    balance_fiat = await fx_svc.convert(balance_usd, currency)
    rates = await fx_svc.get_rates()

    # Staking totals
    staked_atto = str(staking.get("staked", "0"))
    staked_sprx = atto_to_sprx(staked_atto)

    return ApiResponse.ok({
        "address": address,
        "currency": currency.upper(),
        "available": {
            "symbol": "SPRX",
            "balance_atto": balance_atto,
            "balance_sprx": balance_sprx_str,
            "value_usd": balance_usd,
            "value_fiat": balance_fiat,
        },
        "staked": {
            "symbol": "SPRX",
            "balance_atto": staked_atto,
            "balance_sprx": staked_sprx,
        },
        "delegations": delegations,
        "nonce": account.get("nonce", 0),
        "sprx_price_usd": sprx_price_usd,
        "fx_rates": rates,
    })
