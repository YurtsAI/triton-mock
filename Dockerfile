FROM rust:1.75 AS builder

WORKDIR /build
COPY . /build

RUN apt-get update \
  && apt-get install --no-install-recommends -y \
  protobuf-compiler=3.21.12-3 libprotobuf-dev=3.21.12-3 \
  && cargo build --release

FROM debian:bookworm-slim

WORKDIR /work

RUN mkdir /app
COPY --from=builder /build/target/release/triton-mock /app/

EXPOSE 8002 8003 8004 8005 8006 8007

ENTRYPOINT ["/app/triton-mock"]
