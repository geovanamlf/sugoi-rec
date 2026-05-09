from alembic import op
import sqlalchemy as sa

revision = '52328ea024ba'
down_revision = '5e36da4de94f'
branch_labels = None
depends_on = None


def upgrade() -> None:
    op.create_table(
        "recommendation_cache",
        sa.Column("id", sa.Integer(), primary_key=True, index=True),
        sa.Column("user_id", sa.Integer(), sa.ForeignKey("users.id"), nullable=False, unique=True, index=True),
        sa.Column("data", sa.Text(), nullable=False),
        sa.Column("cached_at", sa.DateTime(), nullable=False),
    )


def downgrade() -> None:
    op.drop_table("recommendation_cache")