# Server Configuration Baseline

This baseline keeps proxy behavior deterministic and auditable.

## Required flags/env
- `--endpoint` / `CORTEX_ENDPOINT`: RMVM gRPC endpoint.
- `--planner-mode` / `CORTEX_PLANNER_MODE`: `openai`, `byo`, or `fallback`.
- `CORTEX_BRAIN_SECRET`: brain encryption key env var.

## Recommended limits
- HTTP request body max: 1 MiB.
- gRPC max inbound message: 4 MiB.
- Planner HTTP timeout: 30s (`CORTEX_PLANNER_TIMEOUT_SECS`).
- End-to-end request timeout: 60s (proxy server level).
- Max concurrent requests per process: 100.

## Determinism requirements
- Build with pinned Rust toolchain (`rust-toolchain.toml`).
- Use `cargo build --locked` in CI and releases.
- Keep `core_version.lock` pinned to one core commit.
- Enforce `scripts/check_core_proto_checksums.ps1` in CI.
- Enforce `scripts/check_dependency_manifest_hash.ps1` in CI.

## Operational runbook
1. Start RMVM gRPC (`rmvm-grpc-server`) pinned to `core_version.lock` tag.
2. Start proxy with explicit planner mode.
3. Verify `GET /healthz` returns `ok`.
4. Send golden `POST /v1/chat/completions` request.
5. Confirm `X-Cortex-Status`, proof headers, and response schema.
