project_name := "hemrs"

default: test

check:
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

sqlx_prepare_check:
    docker compose -f docker-compose-test.yaml up --wait
    cargo sqlx migrate run --source backend/migrations
    cargo sqlx prepare --workspace --check
    docker compose -f docker-compose-test.yaml down

docker_builder:
    docker buildx create --name builder --platform linux/amd64

docker_login:
    docker login ghcr.io -u Frixxie -p "$GITHUB_TOKEN"

container: docker_builder docker_login
    docker buildx build -t ghcr.io/frixxie/{{ project_name }}:latest . --builder builder --push

container_tagged dockertag: docker_builder docker_login
    docker buildx build -t ghcr.io/frixxie/{{ project_name }}:{{ dockertag }} . --builder builder --push
