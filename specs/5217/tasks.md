# Issue #5217 Tasks

- Issue: #5217
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): add `docs_contract_matrix_wave2_migration_contract.rs` and run targeted test before harness/deletions.
- T2 (Implementation/GREEN): add `docs_contract_matrix_wave2_harness.rs` with all wave2 case markers.
- T3 (Implementation/GREEN): remove wave2 singleton docs test files after matrix parity is in place.
- T4 (Implementation/GREEN): rewire lane wrappers/selector references from retired singleton tests to the wave2 harness without increasing shell LOC.
- T5 (Verification): run targeted matrix/migration tests, wrapper regression scripts, selector regression script, and clippy/fmt.
- T6 (Process): update issue logs, set spec status `Implemented`, and prepare PR with AC mapping and TDD evidence.
