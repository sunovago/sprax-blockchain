"""Add tap_mining_activity table

Revision ID: 0006
Revises: 0005
Create Date: 2026-08-24

"""
import uuid
from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects.postgresql import UUID

# revision identifiers, used by Alembic.
revision: str = '0006'
down_revision: str | None = '0005'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        'tap_mining_activity',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('tap_miner_id', UUID(as_uuid=True), sa.ForeignKey('tap_miners.id', ondelete='CASCADE'), nullable=False),
        sa.Column('event_type', sa.String(30), nullable=False),
        sa.Column('points_delta', sa.BigInteger(), server_default='0'),
        sa.Column('detail', sa.String(200), nullable=True),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )
    op.create_index('ix_tap_mining_activity_tap_miner_id', 'tap_mining_activity', ['tap_miner_id'])
    op.create_index('ix_tap_mining_activity_created_at', 'tap_mining_activity', ['created_at'])


def downgrade() -> None:
    op.drop_index('ix_tap_mining_activity_created_at', table_name='tap_mining_activity')
    op.drop_index('ix_tap_mining_activity_tap_miner_id', table_name='tap_mining_activity')
    op.drop_table('tap_mining_activity')
