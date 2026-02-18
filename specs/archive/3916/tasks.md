# Issue #3916 Tasks

- Issue: `#3916`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add fallback-violation and missing-marker rejection tests.
- T2 (Green): implement deterministic policy checker helper behavior.
- T3 (Verify): run scoped policy-contract tests.

## Verification Commands
- `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_rejects_fallback_secret_violation_reason_code -- --exact --nocapture`
- `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_rejects_missing_required_lifecycle_markers -- --exact --nocapture`
- `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_accepts_complete_marker_set_without_fallback_violation -- --exact --nocapture`

## Completion Evidence
- fallback-secret and lifecycle-marker policy checks fail closed.
