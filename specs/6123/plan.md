# Plan: Issue #6123

## Approach
1. Add an internal deterministic payload-hash fingerprint encoder using hex encoding of trimmed payload-hash bytes.
2. Replace length-based identity suffixes in `deterministic_runtime_commit_id` and `deterministic_runtime_commit_idempotency_key`.
3. Update runtime request identity unit/contract tests to assert new value-fingerprint behavior and non-collision.
4. Run `kamn-kolme` fmt/clippy/tests.

## Affected Modules
- `crates/kamn-kolme/src/runtime_request_identity_policy.rs`
- `crates/kamn-kolme/tests/runtime_request_identity_policy_contracts.rs`
- `docs/foundation/kolme-runtime-commit-client.md` (regression marker wording)

## Risks
- Risk: external callers may rely on old length-based suffix format.
  - Mitigation: keep prefix and field ordering stable; update tests/docs to lock new deterministic format.

## Interfaces/Contracts
- Public function signatures unchanged.
- Deterministic identity string output format changes in final suffix component.
