# Tasks: Issue #5861 - Service API Relay Spool to Daemon Drain Integration

- Issue: #5861
- Spec: `specs/5861/spec.md`
- Plan: `specs/5861/plan.md`
- Status: Implemented
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (Unit/Integration, RED): add failing tests for relay spool enqueue/drain contracts.
- T2 (Implementation): add shared relay spool path + append/drain helpers.
- T3 (Implementation): wire Service API send handler to enqueue recipient relay entries.
- T4 (Implementation): wire daemon runtime to drain relay spool entries with deterministic cleanup/logging.
- T5 (Regression): run existing mailbox/delivery route tests to ensure response compatibility.
- T6 (Verify): run scoped `cargo test`, `cargo clippy -p kamn-node --tests -- -D warnings`, and `cargo fmt --check`.

## Tier Mapping
| Task | Tier(s) |
|---|---|
| T1 | Unit, Integration, Regression, Conformance |
| T2 | Unit |
| T3 | Integration, Functional |
| T4 | Integration, Regression |
| T5 | Functional, Regression |
| T6 | Verify |
