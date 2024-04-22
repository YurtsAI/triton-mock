# triton-mock

## Overview

A proof-of-concept mock server for the [NVIDIA Triton Inference Server](https://github.com/triton-inference-server/server).  It operates in two modes:

1. Replay mode:  In this mode, the mock server replays the requests it has seen before.

2. Record mode:  In this mode, the mock server records the requests it sees and saves them to disk.

The mock server utilizes the [gRPC definitions](https://github.com/triton-inference-server/common/tree/main/protobuf) from the Triton Inference Server.

To run in recording mode:

```bash
RUST_LOG=debug cargo run --release -- --remote-host 0.0.0.0 --record
```

This requires a real Triton Inference Server running on ports `8302-8307`.  Right now the mapping of model names to ports is hard-coded in [`src/main.rs`](https://github.com/YurtsAI/triton-mock/blob/main/src/main.rs#L57).

These recordings can be replayed using:

```bash
RUST_LOG=debug cargo run --release -- --remote-host 0.0.0.0
```

## Releasing

See [`PUBLISHING.md`](./PUBLISHING.md)
