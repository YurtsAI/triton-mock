ORG_NAME := yurtsai
APP_NAME := triton-mock
VERSION := $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "$(APP_NAME)") | .version')
DOCKER_IMAGE := ghcr.io/$(ORG_NAME)/$(APP_NAME):$(VERSION)

github-token-scope:
	gh auth refresh -s write:packages

docker-build:
	docker build -t $(DOCKER_IMAGE) .

docker-push:
	gh auth token | docker login ghcr.io --username github --password-stdin
	docker push $(DOCKER_IMAGE)

docker-run:
	docker run \
		--name $(APP_NAME) \
		--env RUST_LOG=debug \
		--publish 8002:8002 \
		--publish 8003:8003 \
		--publish 8004:8004 \
		--publish 8005:8005 \
		--publish 8006:8006 \
		--publish 8007:8007 \
		--rm \
		--interactive \
		--tty \
		--volume $(PWD):/work \
		$(DOCKER_IMAGE) \
		$(ARGS)

docker-stop:
	docker stop \
		$(shell docker ps -q --filter "name=$(APP_NAME)") \
		--signal SIGINT
