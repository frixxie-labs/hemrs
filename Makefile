PROJECT_NAME=hemrs

all: test

build:
	cargo check --verbose
	cargo b --verbose

test: build
	cargo t --verbose

docker_builder:
	docker buildx create --name builder --platform linux/amd64,linux/arm64

docker_login:
	docker login ghcr.io -u Frixxie -p $(GITHUB_TOKEN)

container: test docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):latest . --platform linux/amd64,linux/arm64 --builder builder --push

container_tagged: test docker_builder docker_login
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG) . --platform linux/amd64,linux/arm64 --builder builder --push
