PROJECT_NAME=hemrs

all: test

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

docker_builder:
	docker buildx create --name builder --platform linux/amd64

docker_login:
	docker login ghcr.io -u Frixxie -p $(GITHUB_TOKEN)

container: docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):latest . --builder builder --push

container_tagged: docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG) . --builder builder --push
