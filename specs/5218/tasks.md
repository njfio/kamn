# Issue #5218 Tasks

- Issue: #5218
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): add `crates/kamn-core/tests/shell_test_surface_migration_wave2.rs` with inventory + wiring expectations that fail before CI/docs rewiring and deletions.
- T2 (Implementation/GREEN): implement Rust parity assertions for Makefile command-surface, Makefile dry-run execution, and quarantine wrapper behavior.
- T3 (Implementation/GREEN): update `scripts/ci/test_ci_tools.sh` fast/full paths and enforcing contracts/docs to require `cargo test -p kamn-core --test shell_test_surface_migration_wave2`.
- T4 (Deletion/GREEN): delete wave-2 wrappers and update superseded deletion manifest entries for this wave.
- T5 (Verification): run targeted cargo tests plus `test_ci_tools_command_surface_contract.sh`, `test_ci_strategy_contract.sh`, `test_readme_contract.sh`, `test_check_superseded_script_deletion_manifest.sh`, `test_check_stale_script_references.sh`, and ratio guardrail checks.
- T6 (Process): issue progress logs, PR AC/test mapping, shell-surface DoD markers, and status updates.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | helper parsing/assertion helpers inside Rust migration suite |
| Functional | Rust wave-2 suite parity checks for all 3 migrated wrappers |
| Conformance | inventory deletion checks + CI/docs command-surface markers |
| Integration | `scripts/ci/test_ci_tools.sh` invocation of new Rust lane |
| Regression | stale-reference and superseded-manifest checks |
