# Server Configuration Baseline

This baseline keeps proxy behavior deterministic and auditable.

## Required flags/env
- `cortex setup`: writes provider/runtime config and key references.
- `cortex up`: starts managed RMVM + proxy (or reuses external RMVM).
- `CORTEX_BRAIN_SECRET`: brain encryption key env var (auto-hydrated by setup/up in v1).
- `CORTEX_PLANNER_MODE`: `openai`, `byo`, or `fallback`.

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
1. Run `cortex setup` once (interactive or non-interactive).
2. Run `cortex up`.
3. If using external RMVM, set `--rmvm-endpoint` during setup/up.
4. Verify `GET /healthz` returns `ok`.
5. Send golden `POST /v1/chat/completions` request.
6. Confirm `X-Cortex-Status`, proof headers, and response schema.
