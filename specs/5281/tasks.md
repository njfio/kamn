# Issue #5281 Tasks

- Issue: #5281
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing websocket integration test for presence-mode success payload projection.
- [x] T2 (Tests/RED): add failing websocket regression tests for unsupported mode and cross-owner fail-closed behavior.
- [x] T3 (Implementation/GREEN): add websocket presence-mode header contract parsing and deterministic validation.
- [x] T4 (Implementation/GREEN): wire service API websocket route to M9 gateway bridge presence projection.
- [x] T5 (Implementation/GREEN): preserve default websocket state-transition path and ensure compatibility.
- [x] T6 (Verification): run fmt, strict clippy, and targeted websocket endpoint tests.
- [x] T7 (Process): update issue/docs/spec status and closure markers with measured shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | presence-mode header parsing and reason-code mapping |
| Functional | bridge-derived presence payload projection |
| Integration | websocket route + auth middleware + M9 gateway bridge composition |
| Regression | unsupported mode and cross-owner fail-closed paths |
| Performance | N/A (contract projection and route parsing slice) |
