"""Task-claim verification for the two non-manual task types."""
import httpx

from app.core.config import settings
from app.core.logging import logger
from app.db.models.tap_mining import TapMiner
from app.modules.tap_mining.levels import get_level

TELEGRAM_MEMBER_STATUSES = ("member", "administrator", "creator")

MILESTONE_KINDS = ("level", "checkin_streak", "referrals")


async def check_telegram_membership(chat_id: str, telegram_user_id: int) -> bool:
    if not settings.telegram_bot_token or not chat_id:
        return False
    url = f"https://api.telegram.org/bot{settings.telegram_bot_token}/getChatMember"
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            resp = await client.get(url, params={"chat_id": chat_id, "user_id": telegram_user_id})
            data = resp.json()
    except (httpx.HTTPError, ValueError) as e:
        logger.warning("task_verify.telegram_membership_check_failed", chat_id=chat_id, error=str(e))
        return False

    if not data.get("ok"):
        return False
    return data.get("result", {}).get("status") in TELEGRAM_MEMBER_STATUSES


def check_milestone(verify_target: str, verify_value: int, miner: TapMiner, referred_count: int) -> bool:
    if verify_target == "level":
        return get_level(miner.points)["index"] >= verify_value
    if verify_target == "checkin_streak":
        return miner.checkin_streak >= verify_value
    if verify_target == "referrals":
        return referred_count >= verify_value
    return False
