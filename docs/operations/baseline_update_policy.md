# Baseline Update Policy

## When updates are allowed
- Core dependency bump: only via PR updating `core_version.lock`.
- Proto checksum change: only with matching core tag/commit update.
- Dependency manifest hash change: only with reviewed Cargo graph changes.

## Required update steps
1. Update `core_version.lock` (`core_tag`, `core_commit`, checksums, core dependency hash).
2. Refresh local core mirrors:
   - `third_party/core-proto/cortex_rmvm_v3_1.proto`
   - `third_party/core-proto/cortex_rmvm_v3_1_service.proto`
   - `third_party/core/dependency-manifest.sha256`
3. Update workspace dependencies in `Cargo.toml` to the same `core_commit`.
4. Run:
   - `pwsh scripts/check_core_version_lock.ps1`
   - `pwsh scripts/check_core_proto_checksums.ps1`
   - `pwsh scripts/update_dependency_manifest_hash.ps1`
   - `cargo test --locked`
5. Commit `.ci/dependency-manifest.sha256` update in same PR.
6. Include compatibility note in changelog.

## Prohibited
- Floating branch dependencies to core.
- Merging dependency updates without lock/hash updates.
- Publishing release when any lock/hash check is failing.
