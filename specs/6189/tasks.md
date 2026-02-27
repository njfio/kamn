# Tasks: Issue 6189 - Atomic Service API State Writes

- Issue: #6189
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing unit tests for atomic writer success/failure contracts (`C-01`, `C-02`).
- [x] T2 (GREEN): implement atomic state write helper and wire `persist` to it (`C-01`, `C-02`).
- [x] T3 (REGRESSION): run existing service-api persistence integration tests (`C-03`).
- [x] T4 (VERIFY): run `cargo fmt --check`, scoped `clippy`, and targeted `kamn-node` test lane.

## Test Tier Mapping

- Unit: atomic helper tests.
- Functional/Integration: service-api message-store persistence behavior tests.
- Regression: existing persistence tests remain green after implementation swap.
