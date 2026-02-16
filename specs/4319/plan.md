# Issue #4319 Plan

- Issue: `#4319`
- Status: `InProgress`

## Approach
- Add a dedicated peer transport integrity/timeout test suite to keep this subtask isolated.
- Cover required categories (`Unit`, `Functional`, `Integration`, `Regression`, `Performance`) in one deterministic matrix test file.
- Update `docs/planning/kolme-devnet-ops.md` with explicit peer-integrity drift and timeout classification markers.

## Affected Modules
- `crates/kamn-core/tests/p2p_peer_integrity_drift_timeout.rs` (new)
- `docs/planning/kolme-devnet-ops.md`

## Risks and Mitigations
- Risk: flaky performance gate in shared CI runners.
- Mitigation: use a generous local budget threshold and deterministic loop size.
- Risk: overlap with existing reconnect tests causes duplication drift.
- Mitigation: target this suite to integrity-drift + timeout-misclassification gaps only.

## Interface Contract
- No production API or protocol behavior changes.
- Test-only and docs-only delta for this subtask.

## ADR
- Not required (no architecture/protocol/dependency change).
