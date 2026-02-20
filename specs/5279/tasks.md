# Issue #5279 Tasks

- Issue: #5279
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing tests for deterministic M9 dispatch gateway envelope projection across supported transports.
- [x] T2 (Tests/RED): add failing tests for deterministic presence connect/query gateway envelope projection.
- [x] T3 (Tests/RED): add failing regression tests for unsupported transport and owner-scope/presence-visibility fail-closed branches.
- [x] T4 (Implementation/GREEN): add M9 gateway bridge transport/envelope/error types with stable reason markers.
- [x] T5 (Implementation/GREEN): add dispatch and presence projection functions wiring through M9 contracts.
- [x] T6 (Implementation/GREEN): export new gateway bridge APIs in `lib.rs`.
- [x] T7 (Verification): run fmt, strict clippy, and targeted bridge tests.
- [x] T8 (Process): update issue/docs/spec status and closure markers with measured shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | transport normalization and envelope field projection |
| Functional | deterministic dispatch and presence gateway event projection |
| Integration | M9 registry + channel/anti-spam composition through bridge layer |
| Regression | unsupported transport and scope/visibility fail-closed branches |
| Performance | N/A (deterministic projection slice) |
