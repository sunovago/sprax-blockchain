SPRX Backend - Alembic Migrations

Usage:
  alembic upgrade head       # Apply all migrations
  alembic revision --autogenerate -m "description"   # Generate new migration
  alembic downgrade -1       # Rollback one migration
  alembic history            # View migration history

Never manually alter production DB schema outside of controlled migrations.
