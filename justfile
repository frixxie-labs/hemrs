project_name := "hemrs"
set export

SQLX_OFFLINE := "1"

default: test

check: sqlx_prepare
    cargo check

build: check
    cargo build

test: build
    docker compose -f docker-compose-test.yaml up --wait
    cargo test
    docker compose -f docker-compose-test.yaml down

integration_test: build
    docker compose up --build --wait
    cargo install sqlx-cli hurl
    sqlx migrate run --source backend/migrations
    hurl -v backend/backend.hurl
    docker compose down

sqlx_prepare:
    docker compose -f docker-compose-test.yaml up --wait
    cargo sqlx migrate run --source backend/migrations
    cargo sqlx prepare --workspace
    docker compose -f docker-compose-test.yaml down

