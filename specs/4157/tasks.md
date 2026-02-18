# Tasks — Issue #4157

- Status: Implemented

- [x] T1 (Red): add a failing docs-contract test for missing R27.19 closure markers.
- [x] T2 (Green): add R27.19 rehearsal/rollback closure markers to the production next-steps plan.
- [x] T3 (Regression): run targeted docs-contract tests and verify pass.
- [x] T4 (Closeout): set spec/plan/tasks status to Implemented and record completion evidence.

## Completion Evidence

- `cargo test -p kamn-core --test rehearsal_rollback_governance_docs -- --nocapture`
- `bash scripts/ci/test_production_service_next_steps_contract.sh`
