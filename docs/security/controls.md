# Security Controls and Threats

## Threat model
- Brain exfiltration: encrypted brain files copied off-host.
- Key leakage: env var or local secret exfiltration.
- Prompt injection via metadata: planner output attempts unsafe refs/sinks.
- Replay attacks: old requests reused against current brain state.

## Controls
- Encryption at rest: Argon2id + XChaCha20-Poly1305.
- Manifest signing: Ed25519 signatures.
- Plan guard: manifest-ref validation before execute.
- Taint/trust enforcement remains in RMVM core.
- Audit trail for attach/forget/merge operations.

## Required tests
- Invalid `X-Cortex-Plan` base64 and malformed JSON rejected.
- Unknown handle/selector refs rejected pre-execute.
- STALL/REJECTED mapping returns deterministic status and headers.
- Checksum lock scripts fail when core proto drifts.

## Release controls
- Attach SHA-256 checksums to every binary.
- Attach cosign signatures and certificates.
- Attach SBOM for every release binary.
