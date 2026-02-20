# Issue #5275 Tasks

- Issue: #5275
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing tests for deterministic AGE write/read projection from M6 inputs.
- [x] T2 (Tests/RED): add failing regression tests for extension-unavailable and invalid relation-kind fail-closed branches.
- [x] T3 (Implementation/GREEN): add AGE projection kinds/config/request types and reason-code constants.
- [x] T4 (Implementation/GREEN): add AGE projection functions and bridge error variants.
- [x] T5 (Implementation/GREEN): export new AGE bridge APIs in `lib.rs`.
- [x] T6 (Verification): run fmt, strict clippy, and targeted bridge tests.
- [x] T7 (Process): update issue/docs/spec status and closure markers with measured shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | relation and projection input validation |
| Functional | deterministic AGE write/read descriptor projection |
| Integration | M6 contract outputs composed into bridge descriptors |
| Regression | extension unavailable and invalid relation-kind fail-closed paths |
| Performance | N/A (pure deterministic projection slice) |

## Closure Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: 336
- shell_to_rust_ratio_delta_actual: -0.001663
- shell_surface_ratio_target_status: improved
