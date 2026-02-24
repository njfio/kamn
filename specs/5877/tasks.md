# Tasks: Issue #5877 - Production Expect Checker Test-Only Path Hardening

- Issue: #5877
- Spec: `specs/5877/spec.md`
- Plan: `specs/5877/plan.md`
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (Red, Conformance): Add/execute checker regression proving current false-positive on `src/**/tests.rs` path classification.
- T2 (Green, Functional): Update checker exclusion logic for `tests.rs` test-only path shape.
- T3 (Regression): Run `scripts/ci/test_check_no_production_expect.sh` and confirm deterministic outputs unchanged for true production violations.
- T4 (Verify): Run targeted crate tests for docs/contract lanes touched in this stream and capture evidence.
