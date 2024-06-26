ORG_NAME := yurtsai
APP_NAME := triton-mock
VERSION := $(shell cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "$(APP_NAME)") | .version')
DOCKER_IMAGE := ghcr.io/$(ORG_NAME)/$(APP_NAME):$(VERSION)

github-token-scope:
	gh auth refresh -s write:packages

docker-build:
	docker run --rm -v $(PWD):/work -w /work cgr.dev/chainguard/apko \
		build triton-mock-base.yaml triton-mock-base:latest triton-mock-base.tar
	docker load -i triton-mock-base.tar
	docker build -t $(DOCKER_IMAGE) .
	docker tag $(DOCKER_IMAGE) $(APP_NAME):latest

docker-push:
	gh auth token | docker login ghcr.io --username github --password-stdin
	docker push $(DOCKER_IMAGE)

docker-run:
	docker run $(DETACH) \
		--name $(APP_NAME) \
		--env RUST_LOG=debug \
		--publish 8002-8007:8002-8007 \
		--rm \
		--interactive \
		--tty \
		--volume $(PWD):/work \
		$(DOCKER_IMAGE) \
		$(ARGS)

docker-start: DETACH=--detach
docker-start: docker-run

docker-stop:
	docker stop \
		$(shell docker ps -q --filter "name=$(APP_NAME)") \
		--signal SIGINT

sync-grpc-protos:
	curl -v -sSL https://raw.githubusercontent.com/triton-inference-server/common/main/protobuf/grpc_service.proto \
		>protos/grpc_service.proto
	curl -v -sSL https://raw.githubusercontent.com/triton-inference-server/common/main/protobuf/health.proto \
		>protos/health.proto
	curl -v -sSL https://raw.githubusercontent.com/triton-inference-server/common/main/protobuf/model_config.proto \
		>protos/model_config.proto

install-deps:
	brew install protobuf

release:
ifeq ($(EXECUTE),)
	cargo release --no-publish $(RELEASE)
else
	cargo release --execute --no-publish $(RELEASE)
endif
