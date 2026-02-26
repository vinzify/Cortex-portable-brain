# Migration from cortex-rmvm Monorepo

This repo is the standalone home for portable brain + proxy UX.

## Old path
- `portable-brain-proxy/` inside `cortex-rmvm`.

## New path
- `cortex-brain` standalone repository root.

## Command mapping
- Old: `cd portable-brain-proxy && cargo test`
- New: `cd cortex-brain && cargo test`

- Old: `cargo run -p cortex-app -- proxy serve ...`
- New: same command in standalone repo.

## Dependency boundary
- Core kernel/proto remain in `cortex-rmvm`.
- Portable repo consumes core at pinned git commit defined in `core_version.lock`.

## Release boundary
- Core releases: RMVM artifacts and SDKs.
- Portable releases: `cortex` binary, signatures, SBOM, installer assets.
