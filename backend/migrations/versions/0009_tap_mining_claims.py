"""Add tap_mining_claims table and payout fields on tap_miners

Revision ID: 0009
Revises: 0008
Create Date: 2026-10-06
"""
from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision: str = '0009'
down_revision: str | None = '0008'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    # 1. Add payout fields to tap_miners
    op.add_column('tap_miners', sa.Column('payout_address', sa.String(length=100), nullable=True))
    op.add_column('tap_miners', sa.Column('last_claimed_at', sa.DateTime(timezone=True), nullable=True))
    op.add_column(
        'tap_miners',
        sa.Column('total_sprx_claimed', sa.BigInteger(), server_default='0', nullable=False),
    )

    # 2. Create tap_mining_claims table
    op.create_table(
        'tap_mining_claims',
        sa.Column('id', postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column(
            'tap_miner_id',
            postgresql.UUID(as_uuid=True),
            sa.ForeignKey('tap_miners.id', ondelete='CASCADE'),
            nullable=False,
            index=True,
        ),
        sa.Column('points_spent', sa.BigInteger(), nullable=False),
        sa.Column('sprx_amount', sa.BigInteger(), nullable=False),
        sa.Column('recipient_address', sa.String(length=100), nullable=False),
        sa.Column('status', sa.String(length=20), server_default='pending', nullable=False, index=True),
        sa.Column('tx_hash', sa.String(length=66), nullable=True),
        sa.Column('error_message', sa.Text(), nullable=True),
        sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.func.now(), nullable=False, index=True),
        sa.Column('completed_at', sa.DateTime(timezone=True), nullable=True),
    )


def downgrade() -> None:
    op.drop_table('tap_mining_claims')
    op.drop_column('tap_miners', 'total_sprx_claimed')
    op.drop_column('tap_miners', 'last_claimed_at')
    op.drop_column('tap_miners', 'payout_address')
