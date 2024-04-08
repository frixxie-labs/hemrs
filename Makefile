PROJECT_NAME=hemrs

all: test

build:
	cargo check --verbose
	cargo b --verbose

test: build
	docker compose -f docker-compose-test.yaml up --wait
	cargo t --verbose
	docker compose -f docker-compose-test.yaml down

integration_test: build
	docker compose up --build --wait
	cargo install sqlx-cli hurl
	sqlx migrate run --source backend/migrations
	hurl -v backend/backend.hurl
	docker compose down

docker_builder:
	docker buildx create --name builder --platform linux/amd64,linux/arm64

docker_login:
	docker login ghcr.io -u Frixxie -p $(GITHUB_TOKEN)

container: docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):latest . --platform linux/amd64,linux/arm64 --builder builder --push

container_tagged: docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG) . --platform linux/amd64,linux/arm64 --builder builder --push
