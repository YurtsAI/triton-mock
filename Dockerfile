ARG BLD_IMAGE=rust:1.75
ARG RUN_IMAGE=gcr.io/distroless/base-debian12
ARG BUSYBOX_IMAGE=busybox:1.35.0-uclibc
ARG TARGET_TRIPLET=x86_64-linux-gnu

FROM ${BUSYBOX_IMAGE} AS busybox-tools

## === PRE-FLIGHT ===

FROM ${RUN_IMAGE} AS preflight

WORKDIR /preflight

COPY --from=busybox-tools \
  /bin/sh /bin/find /bin/tail \
  /bin/

## Gather a list of libs that exist in the runtime image
RUN find /lib/${TARGET_TRIPLET}/ | tail -n +2 >manifest-exclusions.txt

## === BUILD IMAGE ===

FROM ${BLD_IMAGE} AS builder

ARG BIN_NAME=triton-mock
ARG PROTOBUF_VERSION=3.21.12-3
ARG LIBPROTOBUF_VERSION=3.21.12-3

LABEL org.opencontainers.image.source https://github.com/YurtsAI/${BIN_NAME}

RUN apt-get update \
  && apt-get install --no-install-recommends -y \
  protobuf-compiler=${PROTOBUF_VERSION} libprotobuf-dev=${LIBPROTOBUF_VERSION}

WORKDIR /build
COPY . /build

COPY --from=preflight /preflight/manifest-exclusions.txt .

RUN cargo build --release \
  && mkdir libs \
  && ldd target/release/${BIN_NAME} \
  | grep --invert-match --fixed-strings --file=manifest-exclusions.txt \
  | grep --fixed-strings '=>' \
  | sed -e 's@.* => \(.*\) .*@\1@' \
  | xargs --replace={} cp -v {} libs/

## === RUNTIME IMAGE ===

FROM ${RUN_IMAGE}

ARG BIN_NAME=triton-mock
ARG TARGET_TRIPLET=x86_64-linux-gnu

WORKDIR /work

COPY --from=busybox-tools \
  /bin/sh /bin/ls /bin/rm /bin/addgroup /bin/adduser /bin/chown /bin/mkdir \
  /bin/

# create a non-root user
RUN addgroup --system --gid 1000 ${BIN_NAME} \
  && adduser --system --uid 1000 --ingroup ${BIN_NAME} ${BIN_NAME} \
  && chown 1000:1000 /work \
  && rm /bin/sh /bin/ls /bin/rm /bin/addgroup /bin/adduser /bin/chown /bin/mkdir

COPY --from=builder /build/target/release/${BIN_NAME} /app/
COPY --from=builder /build/libs/* /lib/${TARGET_TRIPLET}/

USER ${BIN_NAME}

EXPOSE 8002 8003 8004 8005 8006 8007

ENTRYPOINT ["/app/triton-mock"]
