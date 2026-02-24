# Tasks: Issue #5928 - Task: Bound replay guard memory with TTL/capacity eviction

- Issue: #5928
- Spec: `specs/5928/spec.md`
- Plan: `specs/5928/plan.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED / Conformance): add replay-guard capacity + TTL regression tests in `auth.rs`.
- T2 (GREEN / Implementation): add `ServiceApiReplayGuard` and replace unbounded replay `BTreeSet` usage.
- T3 (Refactor): centralize replay eviction logic in replay-guard helper methods.
- T4 (Regression): run replay-focused service API endpoint tests and auth unit tests.
- T5 (Verify): run `cargo fmt --check` and strict `kamn-node` clippy.
- T6 (Process): set spec/plan/tasks status to Implemented and prepare issue PR evidence.
