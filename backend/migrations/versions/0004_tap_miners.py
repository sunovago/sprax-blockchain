"""Add tap_miners table

Revision ID: 0004
Revises: 0003
Create Date: 2026-08-24

"""
import uuid
from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects.postgresql import UUID

# revision identifiers, used by Alembic.
revision: str = '0004'
down_revision: str | None = '0003'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        'tap_miners',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('telegram_id', sa.BigInteger(), nullable=False),
        sa.Column('telegram_username', sa.String(100), nullable=True),
        sa.Column('telegram_first_name', sa.String(100), nullable=True),
        sa.Column('telegram_last_name', sa.String(100), nullable=True),
        sa.Column('telegram_photo_url', sa.String(500), nullable=True),
        sa.Column('points', sa.BigInteger(), server_default='0'),
        sa.Column('energy', sa.Integer(), server_default='0'),
        sa.Column('energy_last_refill_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('taps_total', sa.BigInteger(), server_default='0'),
        sa.Column('last_tap_at', sa.DateTime(timezone=True), nullable=True),
        sa.Column('is_banned', sa.Boolean(), server_default='false'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_tap_miners_telegram_id', 'tap_miners', ['telegram_id'], unique=True)


def downgrade() -> None:
    op.drop_index('ix_tap_miners_telegram_id', table_name='tap_miners')
    op.drop_table('tap_miners')
