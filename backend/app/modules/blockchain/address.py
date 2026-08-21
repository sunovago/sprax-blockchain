"""
Sprax address validation utilities.
Address format: Bech32 with prefix 'sprax1'
Validator operator format: 'spraxvaloper1'
"""
import re
from app.core.config import settings

ADDRESS_REGEX = re.compile(r'^sprax1[a-zA-Z0-9]{38}$')
VALOPER_REGEX = re.compile(r'^spraxvaloper1[a-zA-Z0-9]{38}$')
TX_HASH_REGEX = re.compile(r'^0x[0-9a-fA-F]{64}$|^[0-9a-fA-F]{64}$')


def is_valid_address(address: str) -> bool:
    return bool(ADDRESS_REGEX.match(address))


def is_valid_valoper(address: str) -> bool:
    return bool(VALOPER_REGEX.match(address))


def is_valid_tx_hash(hash_str: str) -> bool:
    return bool(TX_HASH_REGEX.match(hash_str))


def is_block_height(value: str) -> bool:
    try:
        int(value)
        return True
    except (ValueError, TypeError):
        return False


def normalize_address(address: str) -> str:
    return address.strip().lower()


def atto_to_sprx(atto: str | int) -> str:
    """Convert atto-SPRX (10^18) to human-readable SPRX string."""
    atto_int = int(atto)
    whole = atto_int // 10**18
    frac = atto_int % 10**18
    frac_str = str(frac).zfill(18).rstrip('0')
    if frac_str:
        return f"{whole}.{frac_str}"
    return str(whole)


def sprx_to_atto(sprx: str | float) -> str:
    """Convert SPRX to atto-SPRX string. Use only for display, not financial calcs."""
    from decimal import Decimal
    d = Decimal(str(sprx))
    return str(int(d * Decimal(10**18)))
