# Tasks: Issue #5924 - Task: Replace kamn-core wipe_bytes loop with compiler-safe zeroization

- Issue: #5924
- Spec: `specs/5924/spec.md`
- Plan: `specs/5924/plan.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED / Conformance): added `regression_issue_5924_signature_profile_wipe_bytes_uses_zeroize_trait` and failure-path regression for invalid private key redaction.
- T2 (GREEN / Implementation): replaced manual `wipe_bytes` byte loop with `bytes.zeroize()` in `signature_profile`.
- T3 (Refactor): kept wipe helper centralized and reused across all private-key decode paths.
- T4 (Regression): added `integration_issue_5924_service_auth_round_trip_remains_valid` and executed targeted regression tests.
- T5 (Verify): ran `cargo fmt --check`, strict `kamn-core` clippy, and scoped issue test commands.
- T6 (Process): updated `spec/plan/tasks` status to Implemented for issue lifecycle closure.
