"""Add tap_mining_tasks and tap_mining_task_completions tables

Revision ID: 0007
Revises: 0006
Create Date: 2026-08-24

"""
import uuid
from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects.postgresql import UUID

# revision identifiers, used by Alembic.
revision: str = '0007'
down_revision: str | None = '0006'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        'tap_mining_tasks',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('title', sa.String(200), nullable=False),
        sa.Column('description', sa.String(500), nullable=True),
        sa.Column('task_type', sa.String(20), nullable=False),
        sa.Column('reward_points', sa.BigInteger(), server_default='0'),
        sa.Column('action_url', sa.String(500), nullable=True),
        sa.Column('verify_target', sa.String(200), nullable=True),
        sa.Column('verify_value', sa.Integer(), nullable=True),
        sa.Column('is_active', sa.Boolean(), server_default='true'),
        sa.Column('sort_order', sa.Integer(), server_default='0'),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
    )

    op.create_table(
        'tap_mining_task_completions',
        sa.Column('id', UUID(as_uuid=True), primary_key=True, default=uuid.uuid4),
        sa.Column('tap_miner_id', UUID(as_uuid=True), sa.ForeignKey('tap_miners.id', ondelete='CASCADE'), nullable=False),
        sa.Column('task_id', UUID(as_uuid=True), sa.ForeignKey('tap_mining_tasks.id', ondelete='CASCADE'), nullable=False),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now()),
        sa.UniqueConstraint('tap_miner_id', 'task_id', name='uq_tap_miner_task'),
    )
    op.create_index('ix_tap_mining_task_completions_tap_miner_id', 'tap_mining_task_completions', ['tap_miner_id'])
    op.create_index('ix_tap_mining_task_completions_task_id', 'tap_mining_task_completions', ['task_id'])


def downgrade() -> None:
    op.drop_index('ix_tap_mining_task_completions_task_id', table_name='tap_mining_task_completions')
    op.drop_index('ix_tap_mining_task_completions_tap_miner_id', table_name='tap_mining_task_completions')
    op.drop_table('tap_mining_task_completions')
    op.drop_table('tap_mining_tasks')
