# Issue #5218 Spec

- Title: Task: Execute shell-test migration wave 2 with superseded script deletions
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
The shell test surface remains materially larger than the Rust test surface. Wave-1 migration reduced this gap, but `scripts/ci` still contains low-coupling shell wrappers whose behavior can be validated directly in Rust.

## Scope
In:
- Migrate a bounded wave of 3 low-coupling shell test wrappers into a Rust-native migration suite.
- Delete superseded shell test wrappers after Rust parity coverage is in place.
- Rewire CI/docs command surfaces from deleted wrappers to the Rust wave-2 suite.
- Update superseded-script deletion inventory for this wave and verify stale-reference checks stay green.

Out:
- Runtime feature changes unrelated to migration parity.
- Broad CI architecture changes outside command-surface rewiring for this wave.
- Additional shell-test migration waves beyond the explicit wave-2 inventory.

## Shell-Surface Estimates
- shell_loc_delta_estimate: -145
- rust_loc_delta_estimate: 220
- shell_to_rust_ratio_delta_estimate: -0.0018
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Wave-2 removes the 3 explicit shell wrappers from git-tracked source.
- AC-2: Rust replacement tests validate equivalent contract behavior for each removed wrapper.
- AC-3: CI command-surface/docs contracts reference the Rust wave-2 suite instead of removed wrapper commands.
- AC-4: Superseded-script deletion manifest checks and stale-reference checks pass in the same change set, with shell/rust ratio guardrails remaining compliant.

## Migration Inventory (Wave-2)
1. `scripts/ci/test_makefile_command_surface_contract.sh`
2. `scripts/ci/test_makefile_execution_contract.sh`
3. `scripts/ci/test_run_cargo_test_with_quarantine.sh`

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Wave-2 inventory | All 3 wrappers are deleted and absent from tracked files |
| C-02 | AC-2 | Functional | Rust wave-2 suite execution | Makefile surface/execution and quarantine wrapper behaviors pass via Rust tests |
| C-03 | AC-3 | Regression | `test_ci_tools.sh`, CI strategy doc/tests, README contract | CI/docs command-surface markers require Rust wave-2 suite and no removed wrapper commands |
| C-04 | AC-4 | Conformance | superseded deletion manifest + stale reference checks + shell/rust guardrail checks | All pass in same branch/PR |

## Test Mapping
- C-01/C-02 -> `cargo test -p kamn-core --test shell_test_surface_migration_wave2`
- C-03 -> `bash scripts/ci/test_ci_tools_command_surface_contract.sh`, `bash scripts/ci/test_ci_strategy_contract.sh`, `bash scripts/ci/test_readme_contract.sh`
- C-04 -> `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`, `bash scripts/ci/test_check_stale_script_references.sh`, `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`

## Success Metrics
- Shell test wrapper count decreases from wave-start baseline (`509`) by at least `3`.
- Rust wave-2 replacement suite is executed from CI regression entrypoint (`scripts/ci/test_ci_tools.sh` fast/full paths).
- No stale references to deleted wrappers remain in tracked command surfaces.
