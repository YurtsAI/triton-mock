# triton-mock

## Overview

A mock server for a Triton Inference Server.  It operates in two modes:

1. Replay mode:  In this mode, the mock server replays the requests it has seen before.

2. Record mode:  In this mode, the mock server records the requests it sees and saves them to disk.

This is using the gRPC definitions from the Triton Inference Server.

To run in recording mode:

```bash
RUST_LOG=debug cargo run --release -- --remote-host 0.0.0.0 --record
```

This requires a real Triton Inference Server running on ports `8302-8307`.  Right now the mapping of model names to ports is hard-coded in `src/main.rs`.

These recordings can be replayed using:

```bash
RUST_LOG=debug cargo run --release -- --remote-host 0.0.0.0
```

## Releasing

To release, use the makefile targets:

```bash
make release RELEASE=patch EXECUTE=y
```

Publishing a new docker image is done manually:

```bash
make docker-build docker-publish
```
