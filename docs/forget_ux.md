# Forget UX Guidance

`forget` is suppression, not hard delete.

## Product semantics
- `cortex brain forget` marks matching memory objects as suppressed.
- Suppressed objects remain in encrypted storage and audit history.
- Reads and downstream policy checks must treat suppressed entries as unavailable.

## User-facing copy
- Use language: "Suppressed from future use".
- Avoid language: "Deleted forever" unless hard-delete is implemented.

## Audit visibility
- Every forget call must emit audit entry with subject, predicate, scope, reason, suppressed_count.
- Display a verified confirmation block for forget in UI/log output.

## Degraded lineage behavior
- If suppression breaks a previously valid chain, downstream operations may produce broken-lineage errors.
- UX should explain this as expected safety behavior.
