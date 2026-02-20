# Issue #5273 Tasks

- Issue: #5273
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing bridge tests for deterministic pgvector insert/search projection from M5 inputs.
- [x] T2 (Tests/RED): add failing regression tests for extension-unavailable and dimension-mismatch fail-closed branches.
- [x] T3 (Implementation/GREEN): add pgvector projection kinds/config/request types and reason-code constants.
- [x] T4 (Implementation/GREEN): add pgvector projection functions and bridge error variants.
- [x] T5 (Implementation/GREEN): export new APIs in `lib.rs`.
- [x] T6 (Verification): run fmt, strict clippy, and targeted bridge tests.
- [x] T7 (Process): update issue/docs/spec status and closure markers with measured shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | projection input validation and deterministic marker ordering |
| Functional | insert/search descriptor projection contracts |
| Integration | M5 registry output composed into PG bridge descriptors |
| Regression | extension unavailable and dimension mismatch fail-closed branches |
| Performance | N/A (pure deterministic projection slice) |

## Closure Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: 329
- shell_to_rust_ratio_delta_actual: -0.001634
- shell_surface_ratio_target_status: improved
