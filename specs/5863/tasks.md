# Tasks: Issue #5863 - Daemon Relay Drain Lifecycle Projection

- Issue: #5863
- Spec: `specs/5863/spec.md`
- Plan: `specs/5863/plan.md`
- Status: Draft
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED): add failing tests for daemon projection to `relayed` and recipient `relayed` retrieval to `delivered`.
- T2 (Implementation): add durable state projection helper for relay-drained message IDs.
- T3 (Implementation): wire daemon drain to invoke projection helper and emit deterministic projected-count log marker.
- T4 (Implementation): expand recipient retrieval transition predicate to include `relayed`.
- T5 (Regression): run adjacent recipient mailbox/delivery and relay drain regression lanes.
- T6 (Verify): run scoped tests, `cargo clippy -p kamn-node --tests -- -D warnings`, `cargo fmt --check`.

## Tier Mapping
| Task | Tier(s) |
|---|---|
| T1 | Integration, Regression, Conformance |
| T2 | Unit |
| T3 | Integration |
| T4 | Functional, Regression |
| T5 | Regression, Functional |
| T6 | Verify |
