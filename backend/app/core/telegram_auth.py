"""Telegram WebApp `initData` validation.

Implements Telegram's documented verification algorithm:
https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
"""
import hashlib
import hmac
import json
import time
from urllib.parse import parse_qsl

from app.core.config import settings
from app.core.exceptions import INVALID_TELEGRAM_AUTH, SpraxError


def validate_init_data(init_data: str) -> dict:
    """Verifies the HMAC signature and freshness of a Telegram WebApp `initData`
    string, returning the parsed `user` object on success.

    Raises SpraxError(INVALID_TELEGRAM_AUTH, ..., 401) on any failure.
    """
    if settings.debug and (init_data == "dev_mock" or init_data.startswith("mock:")):
        return {
            "id": 100001,
            "first_name": "Dev",
            "last_name": "Tester",
            "username": "dev_player",
        }

    if not settings.telegram_bot_token:
        raise SpraxError(INVALID_TELEGRAM_AUTH, "Telegram bot token not configured", 401)

    try:
        pairs = parse_qsl(init_data, keep_blank_values=True, strict_parsing=True)
    except ValueError as e:
        raise SpraxError(INVALID_TELEGRAM_AUTH, f"Malformed init_data: {e}", 401)

    data = dict(pairs)
    received_hash = data.pop("hash", None)
    if not received_hash:
        raise SpraxError(INVALID_TELEGRAM_AUTH, "Missing hash in init_data", 401)

    data_check_string = "\n".join(f"{k}={v}" for k, v in sorted(data.items()))
    secret_key = hmac.new(b"WebAppData", settings.telegram_bot_token.encode(), hashlib.sha256).digest()
    computed_hash = hmac.new(secret_key, data_check_string.encode(), hashlib.sha256).hexdigest()

    if not hmac.compare_digest(computed_hash, received_hash):
        raise SpraxError(INVALID_TELEGRAM_AUTH, "Invalid init_data signature", 401)

    auth_date = data.get("auth_date")
    if not auth_date or not auth_date.isdigit():
        raise SpraxError(INVALID_TELEGRAM_AUTH, "Missing or invalid auth_date", 401)
    age = time.time() - int(auth_date)
    if age > settings.telegram_auth_max_age_seconds or age < -60:
        raise SpraxError(INVALID_TELEGRAM_AUTH, "init_data has expired", 401)

    user_raw = data.get("user")
    if not user_raw:
        raise SpraxError(INVALID_TELEGRAM_AUTH, "Missing user in init_data", 401)
    try:
        user = json.loads(user_raw)
    except json.JSONDecodeError as e:
        raise SpraxError(INVALID_TELEGRAM_AUTH, f"Malformed user field: {e}", 401)

    if "id" not in user:
        raise SpraxError(INVALID_TELEGRAM_AUTH, "Missing telegram user id", 401)

    return user
