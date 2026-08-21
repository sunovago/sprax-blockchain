import pytest
from app.modules.blockchain.address import (
    is_valid_address, is_valid_valoper, is_valid_tx_hash,
    is_block_height, atto_to_sprx, sprx_to_atto
)


def test_valid_address():
    assert is_valid_address("sprax1" + "a" * 38) is True

def test_invalid_address_too_short():
    assert is_valid_address("sprax1abc") is False

def test_invalid_address_wrong_prefix():
    assert is_valid_address("cosmos1" + "a" * 38) is False

def test_valid_valoper():
    assert is_valid_valoper("spraxvaloper1" + "a" * 38) is True

def test_invalid_valoper():
    assert is_valid_valoper("spraxvaloper1abc") is False

def test_valid_tx_hash_no_prefix():
    assert is_valid_tx_hash("a" * 64) is True

def test_valid_tx_hash_0x_prefix():
    assert is_valid_tx_hash("0x" + "a" * 64) is True

def test_invalid_tx_hash():
    assert is_valid_tx_hash("0xabc") is False

def test_block_height_valid():
    assert is_block_height("12345") is True

def test_block_height_invalid():
    assert is_block_height("0xabc") is False

def test_atto_to_sprx_one():
    assert atto_to_sprx("1000000000000000000") == "1"

def test_atto_to_sprx_zero():
    assert atto_to_sprx("0") == "0"

def test_sprx_to_atto():
    assert sprx_to_atto("1.0") == "1000000000000000000"

def test_atto_to_sprx_large():
    result = atto_to_sprx("1000000000" + "0" * 18)
    assert result == "1000000000"
