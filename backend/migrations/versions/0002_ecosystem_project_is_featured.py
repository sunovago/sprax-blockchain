"""Add ecosystem_projects.is_featured

Revision ID: 0002
Revises: 0001
Create Date: 2026-08-21

"""
from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

# revision identifiers, used by Alembic.
revision: str = '0002'
down_revision: str | None = '0001'
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.add_column(
        'ecosystem_projects',
        sa.Column('is_featured', sa.Boolean(), server_default='false'),
    )


def downgrade() -> None:
    op.drop_column('ecosystem_projects', 'is_featured')
