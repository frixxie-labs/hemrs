project_name := "hemrs"
set export

SQLX_OFFLINE := "1"
DATABASE_URL := env_var_or_default("DATABASE_URL", "postgresql://postgres:admin@localhost:5432/postgres")

default: ci

# Run the same validation performed before the deploy jobs in CI.
ci: test-plotter test-backend test-frontend

upgrade:
    cargo upgrade --incompatible && cargo update
    cd plotter && uv lock --upgrade
    cd frontend && deno task update && deno update

lint: lint-plotter lint-frontend

lint-plotter:
    uv tool run ruff check plotter/
    uv tool run ruff format --check plotter/

build-plotter: lint-plotter
    cd plotter && uv sync --frozen

test-plotter: build-plotter
    cd plotter && uv run pytest

check:
    cargo check --workspace

build: build-backend

build-backend:
    rustc --version && cargo --version
    cargo build --workspace --verbose

test: test-backend

test-backend: build-backend
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose -f docker-compose-test.yaml up --wait
    trap 'docker compose -f docker-compose-test.yaml down' EXIT
    cargo test --workspace --verbose

lint-frontend:
    cd frontend && deno fmt --check .
    cd frontend && deno lint .

build-frontend: lint-frontend
    cd frontend && deno task check
    cd frontend && deno task build

test-frontend: build-frontend
    cd frontend && deno task test

integration_test: build
    docker compose up --build --wait
    cargo install sqlx-cli hurl
    sqlx migrate run --source backend/migrations
    hurl -v backend/backend.hurl
    docker compose down

sqlx_prepare:
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose -f docker-compose-test.yaml up --wait
    trap 'docker compose -f docker-compose-test.yaml down' EXIT
    cargo sqlx migrate run --source backend/migrations
    cargo sqlx prepare --workspace
