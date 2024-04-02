PROJECT_NAME=hemrs

all: container

build:
	cargo check --verbose
	cargo b --verbose

test: build
	cargo t --verbose

docker_builder: test
	docker buildx create --name builder --platform linux/amd64,linux/arm64

container: test docker_builder
	docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):latest . --platform linux/amd64,linux/arm64 --builder builder

docker_login:
	docker login ghcr.io -u Frixxie -p $(GITHUB_TOKEN)

publish_container: container docker_login
	docker push ghcr.io/frixxie/$(PROJECT_NAME):latest

publish_tagged_container: container docker_login
	docker tag ghcr.io/frixxie/$(PROJECT_NAME):latest ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG)
	docker push ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG)
