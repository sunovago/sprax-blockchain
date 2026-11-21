"""Widen tap_mining_tasks.description to unbounded Text

Revision ID: 0008
Revises: 0007
Create Date: 2026-08-24

"""
from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

# revision identifiers, used by Alembic.
revision: str = '0008'
down_revision: str | None = '0007'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.alter_column('tap_mining_tasks', 'description', type_=sa.Text(), existing_type=sa.String(500))


def downgrade() -> None:
    op.alter_column('tap_mining_tasks', 'description', type_=sa.String(500), existing_type=sa.Text())
