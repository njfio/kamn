# Tasks: Issue #5953 - Strengthen kamn-sdk service HTTPS mutation coverage

- Issue: #5953
- Spec: `specs/5953/spec.md`
- Plan: `specs/5953/plan.md`
- Status: Draft
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): add failing tests for C-01, C-02, and C-03 in `crates/kamn-sdk/tests/service_api_client.rs`.
- T2 (GREEN / Implementation): implement bounded no-progress read handling and ensure flush semantics remain observable.
- T3 (Regression): run targeted `kamn-sdk` tests for stream/response behavior.
- T4 (Mutation): run `cargo mutants --in-diff` for issue scope; remove `MISSED` and `TIMEOUT` outcomes.
- T5 (Verify): run `cargo fmt --check`, strict clippy for `kamn-sdk`, and attach AC/TDD evidence.
- T6 (Process): update issue/PR status and set spec status to Implemented when complete.
