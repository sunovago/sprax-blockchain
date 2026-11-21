"""Tap Mining On-Chain Claim & Payout Dispatch Logic.

Handles conversion of off-chain tap points to on-chain SPRX tokens,
eligibility & cooldown checks, and communication with the SPRX Faucet service.
"""
from __future__ import annotations

from datetime import UTC, datetime
from typing import Any

import httpx

from app.core.config import settings
from app.core.exceptions import (
    CLAIM_COOLDOWN_ACTIVE,
    FAUCET_DISPATCH_FAILED,
    INSUFFICIENT_POINTS,
    INVALID_ADDRESS,
    NO_PAYOUT_ADDRESS,
    SpraxError,
)
from app.core.logging import logger
from app.modules.blockchain.address import is_valid_address, normalize_address


def calculate_payout(points: int, points_per_sprx: int | None = None) -> tuple[int, int]:
    """Calculate the SPRX token payout and the exact points to debit.

    Returns (sprx_amount, points_debited).
    Example: 2,500 points at 1,000 pts/SPRX -> 2 SPRX and 2,000 points debited.
    """
    rate = points_per_sprx or settings.tap_mining_points_per_sprx
    if rate <= 0:
        rate = 1000
    sprx_amount = points // rate
    points_debited = sprx_amount * rate
    return sprx_amount, points_debited


def get_cooldown_remaining(last_claimed_at: datetime | None, now: datetime, cooldown_seconds: int | None = None) -> int:
    """Return the remaining cooldown in seconds before the next payout claim can be submitted."""
    if last_claimed_at is None:
        return 0

    cooldown = cooldown_seconds if cooldown_seconds is not None else settings.tap_mining_claim_cooldown_seconds
    if last_claimed_at.tzinfo is None:
        last_claimed_at = last_claimed_at.replace(tzinfo=UTC)

    elapsed = (now - last_claimed_at).total_seconds()
    if elapsed >= cooldown:
        return 0
    return int(cooldown - elapsed)


def validate_recipient_address(address: str) -> str:
    """Validate and normalize a recipient SPRX address."""
    addr = normalize_address(address)
    if not is_valid_address(addr):
        raise SpraxError(
            INVALID_ADDRESS,
            f"Invalid Sprax Bech32 address: '{address}'. Expected format: 'sprax1...'",
            400,
        )
    return addr


async def dispatch_faucet_payout(
    recipient: str,
    amount: int,
    faucet_url: str | None = None,
    client: httpx.AsyncClient | None = None,
) -> dict[str, Any]:
    """Dispatch token disbursement request to the automated SPRX Faucet service.

    Sends POST /claim to the Faucet daemon, which signs and commits
    the transfer on the blockchain.
    """
    url = (faucet_url or settings.sprax_faucet_url).rstrip("/") + "/claim"
    payload = {"recipient": recipient, "amount": amount}

    close_client = False
    if client is None:
        client = httpx.AsyncClient(timeout=15.0)
        close_client = True

    try:
        response = await client.post(url, json=payload)
        if response.status_code == 429:
            data = response.json() if response.headers.get("content-type", "").startswith("application/json") else {}
            msg = data.get("error", "Faucet rate limit reached. Please retry later.")
            raise SpraxError(CLAIM_COOLDOWN_ACTIVE, msg, 429)

        if response.status_code >= 400:
            err_text = response.text
            try:
                err_json = response.json()
                err_text = err_json.get("error", err_text)
            except Exception:
                pass
            logger.error("faucet_payout_failed", status=response.status_code, error=err_text)
            raise SpraxError(
                FAUCET_DISPATCH_FAILED,
                f"Faucet service rejected payout: {err_text}",
                502,
            )

        data = response.json()
        logger.info(
            "faucet_payout_success",
            recipient=recipient,
            amount=amount,
            tx_hash=data.get("tx_hash"),
        )
        return data

    except httpx.RequestError as exc:
        logger.error("faucet_connection_error", url=url, error=str(exc))
        raise SpraxError(
            FAUCET_DISPATCH_FAILED,
            f"Could not connect to Faucet service at {url}: {exc}",
            503,
        )
    finally:
        if close_client:
            await client.aclose()
