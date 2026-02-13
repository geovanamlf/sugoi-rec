from logging.config import fileConfig
from sqlalchemy import create_engine, pool
from alembic import context

import sys
from pathlib import Path

# Garante que a raiz do projeto esteja no PYTHONPATH
sys.path.append(str(Path(__file__).resolve().parents[1]))

# Importa a config do projeto
from app.core.config import DATABASE_URL
from app.db.base import Base


# Alembic Config object
config = context.config

# Configuração de logging
if config.config_file_name is not None:
    fileConfig(config.config_file_name)

# Metadata do SQLAlchemy
target_metadata = Base.metadata


def run_migrations_offline() -> None:
    """
    Executa migrações em modo offline.
    """
    url = DATABASE_URL

    context.configure(
        url=url,
        target_metadata=target_metadata,
        literal_binds=True,
        dialect_opts={"paramstyle": "named"},
    )

    with context.begin_transaction():
        context.run_migrations()


def run_migrations_online() -> None:
    """
    Executa migrações em modo online.
    """
    connectable = create_engine(
        DATABASE_URL,
        pool_pre_ping=True,
    )

    with connectable.connect() as connection:
        context.configure(
            connection=connection,
            target_metadata=target_metadata,
        )

        with context.begin_transaction():
            context.run_migrations()


if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
