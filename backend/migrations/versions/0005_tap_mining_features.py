"""Add tap mining referral, check-in, and upgrade columns

Revision ID: 0005
Revises: 0004
Create Date: 2026-08-24

"""
from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects.postgresql import UUID

# revision identifiers, used by Alembic.
revision: str = '0005'
down_revision: str | None = '0004'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.add_column('tap_miners', sa.Column('referral_code', sa.String(16), nullable=True))
    op.add_column(
        'tap_miners',
        sa.Column('referred_by_id', UUID(as_uuid=True), sa.ForeignKey('tap_miners.id', ondelete='SET NULL'), nullable=True),
    )
    op.add_column('tap_miners', sa.Column('referral_earnings', sa.BigInteger(), server_default='0'))
    op.add_column('tap_miners', sa.Column('checkin_streak', sa.Integer(), server_default='0'))
    op.add_column('tap_miners', sa.Column('longest_checkin_streak', sa.Integer(), server_default='0'))
    op.add_column('tap_miners', sa.Column('last_checkin_at', sa.DateTime(timezone=True), nullable=True))
    op.add_column('tap_miners', sa.Column('energy_cap_level', sa.Integer(), server_default='0'))
    op.add_column('tap_miners', sa.Column('regen_speed_level', sa.Integer(), server_default='0'))
    op.add_column('tap_miners', sa.Column('multitap_level', sa.Integer(), server_default='0'))

    op.execute("UPDATE tap_miners SET referral_code = upper(substr(replace(id::text, '-', ''), 1, 8))")
    op.alter_column('tap_miners', 'referral_code', nullable=False)

    op.create_index('ix_tap_miners_referral_code', 'tap_miners', ['referral_code'], unique=True)
    op.create_index('ix_tap_miners_referred_by_id', 'tap_miners', ['referred_by_id'])
    op.create_index('ix_tap_miners_points', 'tap_miners', ['points'])


def downgrade() -> None:
    op.drop_index('ix_tap_miners_points', table_name='tap_miners')
    op.drop_index('ix_tap_miners_referred_by_id', table_name='tap_miners')
    op.drop_index('ix_tap_miners_referral_code', table_name='tap_miners')
    op.drop_column('tap_miners', 'multitap_level')
    op.drop_column('tap_miners', 'regen_speed_level')
    op.drop_column('tap_miners', 'energy_cap_level')
    op.drop_column('tap_miners', 'last_checkin_at')
    op.drop_column('tap_miners', 'longest_checkin_streak')
    op.drop_column('tap_miners', 'checkin_streak')
    op.drop_column('tap_miners', 'referral_earnings')
    op.drop_column('tap_miners', 'referred_by_id')
    op.drop_column('tap_miners', 'referral_code')
