"""Initial schema — all SPRX backend tables

Revision ID: 0001
Revises: 
Create Date: 2026-08-20

"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects.postgresql import UUID, JSONB
import uuid

revision: str = '0001'
down_revision: Union[str, None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # ---- users ----
    op.create_table(
        'users',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('address', sa.String(100), nullable=False, unique=True),
        sa.Column('display_name', sa.String(100), nullable=True),
        sa.Column('is_active', sa.Boolean(), nullable=False, server_default='true'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_users_address', 'users', ['address'])

    # ---- sessions ----
    op.create_table(
        'sessions',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('user_id', UUID(as_uuid=True), sa.ForeignKey('users.id', ondelete='CASCADE'), nullable=False),
        sa.Column('refresh_token_hash', sa.String(255), nullable=False, unique=True),
        sa.Column('device_id', sa.String(255), nullable=True),
        sa.Column('ip_address', sa.String(50), nullable=True),
        sa.Column('user_agent', sa.Text(), nullable=True),
        sa.Column('is_active', sa.Boolean(), nullable=False, server_default='true'),
        sa.Column('expires_at', sa.DateTime(timezone=True), nullable=False),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_sessions_user_id', 'sessions', ['user_id'])

    # ---- devices ----
    op.create_table(
        'devices',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('user_id', UUID(as_uuid=True), sa.ForeignKey('users.id', ondelete='CASCADE'), nullable=False),
        sa.Column('device_token', sa.String(512), nullable=False, unique=True),
        sa.Column('platform', sa.String(20), nullable=False),
        sa.Column('push_enabled', sa.Boolean(), server_default='true'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- blocks ----
    op.create_table(
        'blocks',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('height', sa.BigInteger(), nullable=False, unique=True),
        sa.Column('hash', sa.String(66), nullable=False, unique=True),
        sa.Column('parent_hash', sa.String(66), nullable=True),
        sa.Column('proposer', sa.String(100), nullable=True),
        sa.Column('state_root', sa.String(66), nullable=True),
        sa.Column('tx_count', sa.Integer(), server_default='0'),
        sa.Column('gas_used', sa.BigInteger(), server_default='0'),
        sa.Column('timestamp', sa.DateTime(timezone=True), nullable=False),
        sa.Column('indexed_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_blocks_height', 'blocks', ['height'])
    op.create_index('ix_blocks_hash', 'blocks', ['hash'])
    op.create_index('ix_blocks_timestamp', 'blocks', ['timestamp'])

    # ---- transactions ----
    op.create_table(
        'transactions',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('hash', sa.String(66), nullable=False, unique=True),
        sa.Column('block_height', sa.BigInteger(), nullable=False),
        sa.Column('block_hash', sa.String(66), nullable=True),
        sa.Column('sender', sa.String(100), nullable=False),
        sa.Column('recipient', sa.String(100), nullable=True),
        sa.Column('amount', sa.Numeric(38, 0), server_default='0'),
        sa.Column('fee', sa.Numeric(38, 0), server_default='0'),
        sa.Column('gas_limit', sa.BigInteger(), server_default='0'),
        sa.Column('gas_used', sa.BigInteger(), server_default='0'),
        sa.Column('nonce', sa.BigInteger(), server_default='0'),
        sa.Column('memo', sa.Text(), nullable=True),
        sa.Column('tx_type', sa.String(50), server_default='transfer'),
        sa.Column('status', sa.String(20), server_default='success'),
        sa.Column('raw', JSONB(), nullable=True),
        sa.Column('timestamp', sa.DateTime(timezone=True), nullable=False),
        sa.Column('indexed_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_txs_hash', 'transactions', ['hash'])
    op.create_index('ix_txs_block_height', 'transactions', ['block_height'])
    op.create_index('ix_txs_sender', 'transactions', ['sender'])
    op.create_index('ix_txs_recipient', 'transactions', ['recipient'])
    op.create_index('ix_txs_timestamp', 'transactions', ['timestamp'])

    # ---- addresses ----
    op.create_table(
        'addresses',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('address', sa.String(100), nullable=False, unique=True),
        sa.Column('balance', sa.Numeric(38, 0), server_default='0'),
        sa.Column('nonce', sa.BigInteger(), server_default='0'),
        sa.Column('tx_count', sa.Integer(), server_default='0'),
        sa.Column('first_seen_height', sa.BigInteger(), nullable=True),
        sa.Column('last_seen_height', sa.BigInteger(), nullable=True),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_addresses_address', 'addresses', ['address'])

    # ---- transfers ----
    op.create_table(
        'transfers',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('tx_hash', sa.String(66), nullable=False),
        sa.Column('block_height', sa.BigInteger(), nullable=False),
        sa.Column('from_address', sa.String(100), nullable=False),
        sa.Column('to_address', sa.String(100), nullable=False),
        sa.Column('amount', sa.Numeric(38, 0), nullable=False),
        sa.Column('timestamp', sa.DateTime(timezone=True), nullable=False),
    )
    op.create_index('ix_transfers_from', 'transfers', ['from_address'])
    op.create_index('ix_transfers_to', 'transfers', ['to_address'])
    op.create_index('ix_transfers_tx', 'transfers', ['tx_hash'])

    # ---- validators ----
    op.create_table(
        'validators',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('operator_address', sa.String(100), nullable=False, unique=True),
        sa.Column('consensus_pubkey', sa.Text(), nullable=True),
        sa.Column('moniker', sa.String(100), nullable=False),
        sa.Column('identity', sa.String(64), nullable=True),
        sa.Column('website', sa.String(255), nullable=True),
        sa.Column('details', sa.Text(), nullable=True),
        sa.Column('commission_rate', sa.Numeric(6, 4), server_default='0'),
        sa.Column('commission_max_rate', sa.Numeric(6, 4), server_default='0'),
        sa.Column('commission_max_change', sa.Numeric(6, 4), server_default='0'),
        sa.Column('bonded_tokens', sa.Numeric(38, 0), server_default='0'),
        sa.Column('voting_power', sa.BigInteger(), server_default='0'),
        sa.Column('voting_power_share', sa.Numeric(8, 4), server_default='0'),
        sa.Column('status', sa.String(20), server_default='ACTIVE'),
        sa.Column('jailed', sa.Boolean(), server_default='false'),
        sa.Column('uptime_percent', sa.Numeric(5, 2), server_default='100'),
        sa.Column('missed_blocks', sa.BigInteger(), server_default='0'),
        sa.Column('slash_count', sa.Integer(), server_default='0'),
        sa.Column('delegator_count', sa.Integer(), server_default='0'),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_validators_operator_address', 'validators', ['operator_address'])

    # ---- delegations ----
    op.create_table(
        'delegations',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('delegator_address', sa.String(100), nullable=False),
        sa.Column('validator_address', sa.String(100), nullable=False),
        sa.Column('shares', sa.Numeric(38, 8), server_default='0'),
        sa.Column('amount', sa.Numeric(38, 0), server_default='0'),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_delegations_delegator', 'delegations', ['delegator_address'])
    op.create_index('ix_delegations_validator', 'delegations', ['validator_address'])

    # ---- staking_events ----
    op.create_table(
        'staking_events',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('tx_hash', sa.String(66), nullable=False),
        sa.Column('event_type', sa.String(30), nullable=False),
        sa.Column('delegator_address', sa.String(100), nullable=False),
        sa.Column('validator_address', sa.String(100), nullable=True),
        sa.Column('amount', sa.Numeric(38, 0), server_default='0'),
        sa.Column('block_height', sa.BigInteger(), nullable=False),
        sa.Column('timestamp', sa.DateTime(timezone=True), nullable=False),
    )

    # ---- assets ----
    op.create_table(
        'assets',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('symbol', sa.String(30), nullable=False, unique=True),
        sa.Column('name', sa.String(100), nullable=False),
        sa.Column('coingecko_id', sa.String(100), nullable=True),
        sa.Column('market_cap_rank', sa.Integer(), nullable=True),
        sa.Column('is_sprax_native', sa.Boolean(), server_default='false'),
        sa.Column('logo_url', sa.String(500), nullable=True),
        sa.Column('website', sa.String(255), nullable=True),
        sa.Column('description', sa.Text(), nullable=True),
        sa.Column('is_active', sa.Boolean(), server_default='true'),
        sa.Column('metadata', JSONB(), nullable=True),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- market_snapshots ----
    op.create_table(
        'market_snapshots',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('symbol', sa.String(30), nullable=False),
        sa.Column('price_usd', sa.Numeric(30, 10), nullable=False),
        sa.Column('price_change_24h', sa.Numeric(20, 10), server_default='0'),
        sa.Column('price_change_pct_24h', sa.Numeric(10, 4), server_default='0'),
        sa.Column('volume_24h', sa.Numeric(30, 2), server_default='0'),
        sa.Column('market_cap', sa.Numeric(30, 2), nullable=True),
        sa.Column('high_24h', sa.Numeric(30, 10), nullable=True),
        sa.Column('low_24h', sa.Numeric(30, 10), nullable=True),
        sa.Column('circulating_supply', sa.Numeric(30, 2), nullable=True),
        sa.Column('total_supply', sa.Numeric(30, 2), nullable=True),
        sa.Column('sparkline', JSONB(), nullable=True),
        sa.Column('captured_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_market_snapshots_symbol', 'market_snapshots', ['symbol'])

    # ---- watchlists ----
    op.create_table(
        'watchlists',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('user_id', UUID(as_uuid=True), sa.ForeignKey('users.id', ondelete='CASCADE'), nullable=False),
        sa.Column('name', sa.String(100), server_default='My Watchlist'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- watchlist_assets ----
    op.create_table(
        'watchlist_assets',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('watchlist_id', UUID(as_uuid=True), sa.ForeignKey('watchlists.id', ondelete='CASCADE'), nullable=False),
        sa.Column('symbol', sa.String(30), nullable=False),
        sa.Column('added_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.UniqueConstraint('watchlist_id', 'symbol', name='uq_watchlist_asset'),
    )

    # ---- notifications ----
    op.create_table(
        'notifications',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('user_id', UUID(as_uuid=True), sa.ForeignKey('users.id', ondelete='CASCADE'), nullable=False),
        sa.Column('type', sa.String(50), nullable=False),
        sa.Column('title', sa.String(200), nullable=False),
        sa.Column('body', sa.Text(), nullable=False),
        sa.Column('is_read', sa.Boolean(), server_default='false'),
        sa.Column('metadata', JSONB(), nullable=True),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_notifications_user_id', 'notifications', ['user_id'])

    # ---- price_alerts ----
    op.create_table(
        'price_alerts',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('user_id', UUID(as_uuid=True), sa.ForeignKey('users.id', ondelete='CASCADE'), nullable=False),
        sa.Column('symbol', sa.String(30), nullable=False),
        sa.Column('condition', sa.String(10), nullable=False),
        sa.Column('target_price_usd', sa.Numeric(30, 10), nullable=False),
        sa.Column('is_active', sa.Boolean(), server_default='true'),
        sa.Column('triggered_at', sa.DateTime(timezone=True), nullable=True),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- perp_markets ----
    op.create_table(
        'perp_markets',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('symbol', sa.String(30), nullable=False, unique=True),
        sa.Column('base_asset', sa.String(20), nullable=False),
        sa.Column('quote_asset', sa.String(20), nullable=False),
        sa.Column('max_leverage', sa.BigInteger(), server_default='50'),
        sa.Column('min_leverage', sa.BigInteger(), server_default='1'),
        sa.Column('maker_fee', sa.Numeric(6, 4), server_default='0.0002'),
        sa.Column('taker_fee', sa.Numeric(6, 4), server_default='0.0006'),
        sa.Column('mark_price', sa.Numeric(30, 10), server_default='0'),
        sa.Column('index_price', sa.Numeric(30, 10), server_default='0'),
        sa.Column('last_price', sa.Numeric(30, 10), server_default='0'),
        sa.Column('funding_rate', sa.Numeric(12, 8), server_default='0'),
        sa.Column('open_interest', sa.Numeric(30, 8), server_default='0'),
        sa.Column('is_active', sa.Boolean(), server_default='false'),  # disabled by default
        sa.Column('is_testnet_only', sa.Boolean(), server_default='true'),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- orders ----
    op.create_table(
        'orders',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('user_id', UUID(as_uuid=True), sa.ForeignKey('users.id', ondelete='CASCADE'), nullable=False),
        sa.Column('symbol', sa.String(30), nullable=False),
        sa.Column('side', sa.String(10), nullable=False),
        sa.Column('order_type', sa.String(20), nullable=False),
        sa.Column('size', sa.Numeric(30, 8), nullable=False),
        sa.Column('price', sa.Numeric(30, 10), nullable=True),
        sa.Column('leverage', sa.BigInteger(), server_default='1'),
        sa.Column('margin', sa.Numeric(30, 8), server_default='0'),
        sa.Column('status', sa.String(20), server_default='open'),
        sa.Column('is_testnet', sa.Boolean(), server_default='true'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- positions ----
    op.create_table(
        'positions',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('user_id', UUID(as_uuid=True), sa.ForeignKey('users.id', ondelete='CASCADE'), nullable=False),
        sa.Column('symbol', sa.String(30), nullable=False),
        sa.Column('side', sa.String(10), nullable=False),
        sa.Column('size', sa.Numeric(30, 8), nullable=False),
        sa.Column('entry_price', sa.Numeric(30, 10), nullable=False),
        sa.Column('mark_price', sa.Numeric(30, 10), server_default='0'),
        sa.Column('leverage', sa.BigInteger(), server_default='1'),
        sa.Column('margin', sa.Numeric(30, 8), nullable=False),
        sa.Column('liquidation_price', sa.Numeric(30, 10), nullable=True),
        sa.Column('unrealized_pnl', sa.Numeric(20, 8), server_default='0'),
        sa.Column('is_open', sa.Boolean(), server_default='true'),
        sa.Column('is_testnet', sa.Boolean(), server_default='true'),
        sa.Column('opened_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- trades ----
    op.create_table(
        'trades',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('symbol', sa.String(30), nullable=False),
        sa.Column('price', sa.Numeric(30, 10), nullable=False),
        sa.Column('size', sa.Numeric(30, 8), nullable=False),
        sa.Column('side', sa.String(10), nullable=False),
        sa.Column('is_testnet', sa.Boolean(), server_default='true'),
        sa.Column('timestamp', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- funding_events ----
    op.create_table(
        'funding_events',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('symbol', sa.String(30), nullable=False),
        sa.Column('rate', sa.Numeric(12, 8), nullable=False),
        sa.Column('mark_price', sa.Numeric(30, 10), nullable=False),
        sa.Column('timestamp', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- ecosystem_projects ----
    op.create_table(
        'ecosystem_projects',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('project_id', sa.String(100), nullable=False, unique=True),
        sa.Column('name', sa.String(200), nullable=False),
        sa.Column('tag', sa.String(50), nullable=True),
        sa.Column('description', sa.Text(), nullable=True),
        sa.Column('icon_emoji', sa.String(10), nullable=True),
        sa.Column('category', sa.String(50), nullable=False),
        sa.Column('website_url', sa.String(255), nullable=True),
        sa.Column('tvl_usd', sa.Numeric(30, 2), nullable=True),
        sa.Column('active_users_24h', sa.Integer(), nullable=True),
        sa.Column('is_verified', sa.Boolean(), server_default='false'),
        sa.Column('is_active', sa.Boolean(), server_default='true'),
        sa.Column('sort_order', sa.Integer(), server_default='0'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- feature_flags ----
    op.create_table(
        'feature_flags',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('name', sa.String(100), nullable=False, unique=True),
        sa.Column('is_enabled', sa.Boolean(), server_default='false'),
        sa.Column('description', sa.Text(), nullable=True),
        sa.Column('updated_by', sa.String(100), nullable=True),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- admin_users ----
    op.create_table(
        'admin_users',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('username', sa.String(100), nullable=False, unique=True),
        sa.Column('password_hash', sa.String(255), nullable=False),
        sa.Column('role', sa.String(50), server_default='viewer'),
        sa.Column('is_active', sa.Boolean(), server_default='true'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('last_login', sa.DateTime(timezone=True), nullable=True),
    )

    # ---- audit_logs ----
    op.create_table(
        'audit_logs',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('actor', sa.String(100), nullable=False),
        sa.Column('action', sa.String(100), nullable=False),
        sa.Column('resource', sa.String(200), nullable=True),
        sa.Column('result', sa.String(20), server_default='success'),
        sa.Column('request_id', sa.String(36), nullable=True),
        sa.Column('metadata', JSONB(), nullable=True),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_audit_logs_created_at', 'audit_logs', ['created_at'])

    # ---- indexer_state ----
    op.create_table(
        'indexer_state',
        sa.Column('id', sa.Integer(), primary_key=True),
        sa.Column('latest_chain_height', sa.BigInteger(), server_default='0'),
        sa.Column('latest_indexed_height', sa.BigInteger(), server_default='0'),
        sa.Column('lag_blocks', sa.BigInteger(), server_default='0'),
        sa.Column('is_syncing', sa.Boolean(), server_default='false'),
        sa.Column('last_error', sa.String(500), nullable=True),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    # ---- network_stats ----
    op.create_table(
        'network_stats',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('block_height', sa.BigInteger(), nullable=False),
        sa.Column('total_transactions', sa.BigInteger(), server_default='0'),
        sa.Column('active_validators', sa.BigInteger(), server_default='0'),
        sa.Column('total_staked', sa.Numeric(38, 0), server_default='0'),
        sa.Column('tps', sa.Numeric(10, 2), server_default='0'),
        sa.Column('avg_block_time_ms', sa.BigInteger(), server_default='2000'),
        sa.Column('captured_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )


def downgrade() -> None:
    tables = [
        'network_stats', 'indexer_state', 'audit_logs', 'admin_users', 'feature_flags',
        'ecosystem_projects', 'funding_events', 'trades', 'positions', 'orders', 'perp_markets',
        'price_alerts', 'notifications', 'watchlist_assets', 'watchlists', 'market_snapshots', 'assets',
        'staking_events', 'delegations', 'validators', 'transfers', 'addresses', 'transactions', 'blocks',
        'devices', 'sessions', 'users',
    ]
    for t in tables:
        op.drop_table(t)
