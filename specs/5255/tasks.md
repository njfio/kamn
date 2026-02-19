# Issue #5255 Tasks

- Issue: #5255
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Ordered Tasks
- [x] T1 (Tests/RED): add `data_layer_postgres_migration_contract` test with required table/index/policy markers before migration SQL exists.
- [x] T2 (Implementation/GREEN): add baseline migration SQL for six core tables plus deterministic index/RLS marker blocks.
- [x] T3 (Implementation/GREEN): update roadmap/activation-plan/milestone docs with `#5247/#5248/#5255` phase linkage.
- [x] T4 (Regression): run targeted migration contract tests and keep existing data-layer contract suites green.
- [x] T5 (Verification): run `cargo fmt --check` and `cargo clippy -p kamn-core --tests -- -D warnings` for touched surfaces.
- [x] T6 (Process): set spec status to `Implemented`, capture shell/rust deltas, and prepare PR-ready evidence.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | migration marker parsing helpers inside test suite |
| Functional | required migration table/index/policy marker presence |
| Integration | docs+milestone linkage assertions for phase activation |
| Regression | fail-closed marker-removal behavior checks |
| Performance | N/A (schema bootstrap only) |
