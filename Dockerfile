ARG BLD_IMAGE=cgr.dev/chainguard/rust
ARG BLD_IMAGE_TAG=latest-dev

ARG RUN_IMAGE=triton-mock-base
ARG RUN_IMAGE_TAG=latest-amd64

## === BUILD IMAGE ===

FROM ${BLD_IMAGE}:${BLD_IMAGE_TAG} AS builder

ARG PROTOBUF_VERSION=3.25.3-r0

USER root

RUN apk add --no-cache \
  protobuf=${PROTOBUF_VERSION} \
  protobuf-dev=${PROTOBUF_VERSION}

USER nonroot

WORKDIR /build
COPY . /build

RUN cargo build --release && strip target/release/triton-mock

## === RUNTIME IMAGE ===

FROM ${RUN_IMAGE}:${RUN_IMAGE_TAG}

LABEL org.opencontainers.image.source https://github.com/YurtsAI/triton-mock

COPY --from=builder /build/target/release/triton-mock /app/

EXPOSE 8005 8007

ENTRYPOINT ["/app/triton-mock"]
