# Tasks: Issue #5911 - Disable Legacy Baseline-v1 Signature Compatibility in Production Builds

1. T1 (RED): add failing helper-policy tests for non-debug fail-closed compatibility semantics.
2. T2 (GREEN): harden compatibility helper policy in signer backend and transaction modules.
3. T3 (REFACTOR): ensure shared policy semantics stay deterministic across both modules.
4. T4 (VERIFY): run fmt, clippy, signer backend + transaction tests.
5. T5 (REGRESSION): run mutation slice for touched compatibility helpers.
