FROM rust:1.75 AS builder

LABEL org.opencontainers.image.source https://github.com/YurtsAI/triton-mock

ARG PROTOBUF_VERSION=3.21.12-3
ARG LIBPROTOBUF_VERSION=3.21.12-3

RUN apt-get update \
  && apt-get install --no-install-recommends -y \
  protobuf-compiler=${PROTOBUF_VERSION} libprotobuf-dev=${LIBPROTOBUF_VERSION}

WORKDIR /build
COPY . /build

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /work

RUN mkdir /app
COPY --from=builder /build/target/release/triton-mock /app/

EXPOSE 8002 8003 8004 8005 8006 8007

ENTRYPOINT ["/app/triton-mock"]
