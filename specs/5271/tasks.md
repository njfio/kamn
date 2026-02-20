# Issue #5271 Tasks

- Issue: #5271
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add failing reconciliation tests for pending/final/failed and fail-closed mismatch branches.
- [x] T2 (Implementation/GREEN): add finality-observation and reconciliation projection types/functions.
- [x] T3 (Implementation/GREEN): wire reconciliation mapping to follow-up policy + confirmation metadata projection.
- [x] T4 (Regression): add commit-id mismatch and final-without-block-height fail-closed regression coverage.
- [x] T5 (Verification): run fmt, strict clippy, and targeted tests.
- [x] T6 (Process): update issue/docs/spec status and closure markers with measured shell/rust deltas.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | finality observation validation and reconciliation mapping |
| Functional | pending/final/failed deterministic projection |
| Integration | reconciliation coherence with adapter lifecycle persistence projection |
| Regression | tx mismatch and missing block-height fail-closed branches |
| Performance | N/A (projection contract slice only) |

## Closure Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: 346
- shell_to_rust_ratio_delta_actual: -0.001726
- shell_surface_ratio_target_status: improved
