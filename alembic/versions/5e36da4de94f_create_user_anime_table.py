from alembic import op
import sqlalchemy as sa

revision = '5e36da4de94f'
down_revision = 'e2c16e92553e'
branch_labels = None
depends_on = None


def upgrade() -> None:
    op.create_table(
        "user_anime",
        sa.Column("id", sa.Integer(), primary_key=True, index=True),
        sa.Column("user_id", sa.Integer(), sa.ForeignKey("users.id"), nullable=False, index=True),
        sa.Column("anime_id", sa.Integer(), sa.ForeignKey("anime_cache.id"), nullable=False, index=True),
        sa.Column("status", sa.String(20), nullable=False),
        sa.Column("rating", sa.Integer(), nullable=True),
        sa.Column("is_favorite", sa.Boolean(), default=False, nullable=False),
        sa.Column("added_at", sa.DateTime(), nullable=False),
    )


def downgrade() -> None:
    op.drop_table("user_anime")
