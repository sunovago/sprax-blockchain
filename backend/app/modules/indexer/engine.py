"""
Sprax Indexer Engine.
Polls the chain, indexes blocks+transactions+validators into PostgreSQL.
Runs as a background Celery task or standalone async loop.
"""
from __future__ import annotations
import asyncio
from datetime import datetime, timezone
from decimal import Decimal
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, update
from app.modules.blockchain.client import get_chain_client
from app.db.session import async_session_factory
from app.db.models.blockchain import Block, Transaction, Address, Transfer
from app.db.models.validator import Validator, Delegation
from app.db.models.indexer import IndexerState, NetworkStat
from app.core.logging import logger
from app.core.config import settings


class IndexerEngine:
    """Polls Sprax Chain and indexes data into PostgreSQL."""

    def __init__(self):
        self._client = get_chain_client()
        self._running = False

    async def run(self) -> None:
        """Main indexer loop."""
        self._running = True
        logger.info("indexer_started")
        while self._running:
            try:
                await self._index_cycle()
            except Exception as e:
                logger.error("indexer_cycle_error", error=str(e))
            await asyncio.sleep(settings.indexer_poll_interval_seconds)

    async def stop(self) -> None:
        self._running = False

    async def _index_cycle(self) -> None:
        async with async_session_factory() as db:
            state = await self._get_or_create_state(db)
            chain_height = await self._client.get_latest_block_height()
            state.latest_chain_height = chain_height
            state.lag_blocks = chain_height - state.latest_indexed_height

            if state.latest_indexed_height >= chain_height:
                await db.commit()
                return

            start = state.latest_indexed_height + 1
            end = min(start + settings.indexer_batch_size, chain_height + 1)

            for height in range(start, end):
                await self._index_block(db, height)
                state.latest_indexed_height = height
                state.lag_blocks = chain_height - height

            await self._sync_validators(db)
            await self._capture_network_stat(db, chain_height)
            await db.commit()

    async def _index_block(self, db: AsyncSession, height: int) -> None:
        try:
            block_data = await self._client.get_block(height)
            if not block_data:
                return

            block = Block(
                height=height,
                hash=block_data.get("hash", f"0x{height:064x}"),
                parent_hash=block_data.get("parent_hash"),
                proposer=block_data.get("proposer"),
                state_root=block_data.get("state_root"),
                tx_count=len(block_data.get("transactions", [])),
                gas_used=int(block_data.get("gas_used", 0)),
                timestamp=self._parse_ts(block_data.get("timestamp")),
            )
            db.add(block)

            for tx_data in block_data.get("transactions", []):
                await self._index_transaction(db, tx_data, height, block.timestamp)

        except Exception as e:
            logger.error("index_block_error", height=height, error=str(e))

    async def _index_transaction(self, db: AsyncSession, tx_data: dict, height: int, block_ts: datetime) -> None:
        try:
            tx = Transaction(
                hash=tx_data.get("hash", ""),
                block_height=height,
                block_hash=tx_data.get("block_hash"),
                sender=tx_data.get("sender", ""),
                recipient=tx_data.get("recipient"),
                amount=Decimal(str(tx_data.get("amount", "0"))),
                fee=Decimal(str(tx_data.get("fee", "0"))),
                gas_limit=int(tx_data.get("gas_limit", 0)),
                gas_used=int(tx_data.get("gas_used", 0)),
                nonce=int(tx_data.get("nonce", 0)),
                memo=tx_data.get("memo"),
                tx_type=tx_data.get("type", "transfer"),
                status="success" if tx_data.get("success", True) else "failed",
                timestamp=block_ts,
            )
            db.add(tx)

            # Upsert address records
            sender = tx_data.get("sender", "")
            recipient = tx_data.get("recipient")
            if sender:
                await self._touch_address(db, sender, height)
            if recipient:
                await self._touch_address(db, recipient, height)

            # Record transfer
            if recipient and int(tx_data.get("amount", 0)) > 0:
                transfer = Transfer(
                    tx_hash=tx.hash,
                    block_height=height,
                    from_address=sender,
                    to_address=recipient,
                    amount=Decimal(str(tx_data.get("amount", "0"))),
                    timestamp=block_ts,
                )
                db.add(transfer)
        except Exception as e:
            logger.error("index_tx_error", error=str(e))

    async def _touch_address(self, db: AsyncSession, address: str, height: int) -> None:
        """Create or update address record."""
        result = await db.execute(select(Address).where(Address.address == address))
        addr = result.scalar_one_or_none()
        if addr is None:
            addr = Address(
                address=address,
                tx_count=1,
                first_seen_height=height,
                last_seen_height=height,
            )
            db.add(addr)
        else:
            addr.tx_count += 1
            addr.last_seen_height = height

    async def _sync_validators(self, db: AsyncSession) -> None:
        """Sync validator set from chain."""
        try:
            validators = await self._client.get_validators()
            total_power = sum(int(v.get("votingPower", 0)) for v in validators)
            for v_data in validators:
                addr = v_data.get("address", "")
                result = await db.execute(select(Validator).where(Validator.operator_address == addr))
                val = result.scalar_one_or_none()
                power = int(v_data.get("votingPower", 0))
                share = (power / total_power * 100) if total_power > 0 else 0

                if val is None:
                    val = Validator(
                        operator_address=addr,
                        moniker=v_data.get("name", addr[:20]),
                        voting_power=power,
                        voting_power_share=Decimal(str(round(share, 4))),
                        commission_rate=Decimal(str(v_data.get("commissionRate", 0))),
                        status=v_data.get("status", "ACTIVE"),
                    )
                    db.add(val)
                else:
                    val.voting_power = power
                    val.voting_power_share = Decimal(str(round(share, 4)))
                    val.status = v_data.get("status", val.status)
                    val.commission_rate = Decimal(str(v_data.get("commissionRate", float(val.commission_rate))))
        except Exception as e:
            logger.error("sync_validators_error", error=str(e))

    async def _capture_network_stat(self, db: AsyncSession, height: int) -> None:
        result = await db.execute(select(NetworkStat).order_by(NetworkStat.captured_at.desc()).limit(1))
        last = result.scalar_one_or_none()
        v_result = await db.execute(select(Validator).where(Validator.status == "ACTIVE"))
        active_count = len(v_result.scalars().all())

        stat = NetworkStat(
            block_height=height,
            active_validators=active_count,
        )
        db.add(stat)

    async def _get_or_create_state(self, db: AsyncSession) -> IndexerState:
        result = await db.execute(select(IndexerState).where(IndexerState.id == 1))
        state = result.scalar_one_or_none()
        if state is None:
            state = IndexerState(id=1)
            db.add(state)
            await db.flush()
        return state

    @staticmethod
    def _parse_ts(ts: str | int | None) -> datetime:
        if ts is None:
            return datetime.now(timezone.utc)
        if isinstance(ts, int):
            return datetime.fromtimestamp(ts, tz=timezone.utc)
        try:
            return datetime.fromisoformat(ts.replace("Z", "+00:00"))
        except (ValueError, AttributeError):
            return datetime.now(timezone.utc)
