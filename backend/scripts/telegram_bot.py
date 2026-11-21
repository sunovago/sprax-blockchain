"""
Minimal long-polling handler for the tap-mining Telegram bot.

The bot's menu button already opens the Mini App directly (set via the
Bot API's setChatMenuButton, no server needed for that). This script only
exists so the bot also replies to /start in-chat with a "Play" button,
since a bot that never responds to messages looks broken even though the
menu button works fine without it.

Usage: python scripts/telegram_bot.py
"""
import asyncio

import httpx

from app.core.config import settings
from app.core.logging import configure_logging, logger

API_BASE = "https://api.telegram.org"

def build_welcome_text(first_name: str) -> str:
    safe_name = (first_name or "Miner").replace("<", "&lt;").replace(">", "&gt;")
    return (
        f"👋 <b>Hey {safe_name}, Welcome to SPRX Mining!</b> ⚡\n\n"
        f"🚀 <b>Sprax Network Tap-to-Earn Ecosystem</b>\n"
        f"Start mining SPRX points today, level up your rank, and claim real on-chain "
        f"<b>$SPRX tokens</b> directly to your Sprax wallet!\n\n"
        f"🔥 <b>Key Features:</b>\n"
        f"• ⚡ <b>Tap to Mine:</b> Earn points every tap with multitap upgrades.\n"
        f"• 🔋 <b>Energy Tank:</b> Upgrade capacity and recharge rate in the Shop.\n"
        f"• 📅 <b>Daily Streaks:</b> Check in every 24 hours for compounding bonuses.\n"
        f"• 🏆 <b>Leaderboards:</b> Compete from Bronze up to Diamond tier.\n"
        f"• 👥 <b>Squad Referrals:</b> Invite friends and earn 10% of their rewards.\n"
        f"• 💎 <b>Token Claims:</b> Convert mined points to real testnet/mainnet tokens!\n\n"
        f"👇 <i>Tap the button below to launch the game and claim your starting bonus!</i>"
    )

HELP_TEXT = (
    "📖 <b>How to play SPRX Tap Mining:</b>\n\n"
    "1️⃣ Tap <b>⚡ Play SPRX Miner</b> below to open the Mini App.\n"
    "2️⃣ Tap the SPRX core to earn points before your energy runs out.\n"
    "3️⃣ Visit <b>Shop</b> to boost Multitap, Energy Cap, and Regen Speed.\n"
    "4️⃣ Check in daily to build your streak for massive multiplier rewards.\n"
    "5️⃣ Link your <code>sprax1...</code> wallet address in the <b>Profile</b> tab to claim token payouts!\n\n"
    "Need support? Check our docs or join our community group below."
)


async def handle_update(client: httpx.AsyncClient, token: str, update: dict) -> None:
    message = update.get("message")
    if not message or "text" not in message:
        return
    text = message["text"].strip()
    parts = text.split(maxsplit=1)
    command = parts[0].lower()

    chat_id = message["chat"]["id"]
    sender = message.get("from", {})
    first_name = sender.get("first_name", "Miner")

    if not settings.tap_mining_webapp_url:
        await client.post(
            f"{API_BASE}/bot{token}/sendMessage",
            json={
                "chat_id": chat_id,
                "text": "⚠️ Tap mining is temporarily offline — operator needs to set TAP_MINING_WEBAPP_URL.",
            },
        )
        return

    webapp_url = settings.tap_mining_webapp_url
    if command.startswith("/start") and len(parts) > 1 and parts[1]:
        webapp_url = f"{webapp_url}?ref={parts[1].strip()}"

    if command.startswith("/start"):
        keyboard = {
            "inline_keyboard": [
                [{"text": "⚡ 🎮 Launch SPRX Miner", "web_app": {"url": webapp_url}}],
                [
                    {"text": "📢 Official News", "url": "https://t.me/spraxchain"},
                    {"text": "💬 Community Chat", "url": "https://t.me/spraxcommunity"},
                ],
                [
                    {"text": "🌐 Sprax Website", "url": "https://sprax.network"},
                    {"text": "📖 How to Play", "callback_data": "help_info"},
                ],
            ]
        }
        await client.post(
            f"{API_BASE}/bot{token}/sendMessage",
            json={
                "chat_id": chat_id,
                "text": build_welcome_text(first_name),
                "parse_mode": "HTML",
                "reply_markup": keyboard,
            },
        )
    elif command.startswith("/help"):
        keyboard = {
            "inline_keyboard": [
                [{"text": "⚡ 🎮 Launch SPRX Miner", "web_app": {"url": webapp_url}}],
            ]
        }
        await client.post(
            f"{API_BASE}/bot{token}/sendMessage",
            json={
                "chat_id": chat_id,
                "text": HELP_TEXT,
                "parse_mode": "HTML",
                "reply_markup": keyboard,
            },
        )


async def main() -> None:
    configure_logging()
    token = settings.telegram_bot_token
    if not token:
        logger.error("telegram_bot.missing_token")
        return

    offset = 0
    async with httpx.AsyncClient(timeout=35.0) as client:
        logger.info("telegram_bot.polling_started")
        while True:
            try:
                resp = await client.get(f"{API_BASE}/bot{token}/getUpdates", params={"offset": offset, "timeout": 30})
                resp.raise_for_status()
                for update in resp.json().get("result", []):
                    offset = update["update_id"] + 1
                    await handle_update(client, token, update)
            except httpx.HTTPError as e:
                logger.warning("telegram_bot.poll_error", error=str(e))
                await asyncio.sleep(3)


if __name__ == "__main__":
    asyncio.run(main())
