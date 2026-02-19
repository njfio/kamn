# Issue #5189 Spec

- Title: Task: execute shell-vs-rust test surface migration wave 1 and ratio ratchet
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Problem Statement
Shell test wrappers significantly outnumber Rust-native tests in the CI regression surface. Wrapper-only shell tests increase maintenance overhead, increase shell LOC pressure, and keep the shell-vs-rust test surface ratio close to fail thresholds.

## Scope
In:
- Remove at least 20 low-coupling shell test wrappers from `scripts/ci` and `scripts/runtime`.
- Replace wrapper coverage with Rust-native integration/conformance tests in `crates/kamn-core/tests`.
- Add CI-enforced shell-vs-rust test-file ratio non-regression policy with waiver support.
- Update CI command-surface/doc contracts to reference Rust migration lanes instead of removed shell wrappers.

Out:
- Migration of all remaining shell test wrappers (follow-up waves).
- Behavioral changes to underlying policy/checker implementations beyond parity-preserving coverage migration.

## Acceptance Criteria
- AC-1: Wave-1 removes at least 20 shell test wrapper files while preserving deterministic contract coverage.
- AC-2: Rust-native replacement coverage exists for every removed shell lane and is executed by CI.
- AC-3: CI enforces shell-vs-rust test-file ratio non-regression, with fail-closed behavior unless an explicit waiver is provided.

## Migration Inventory (Wave-1)
1. scripts/ci/test_block_reconciliation_partition_rejoin_ci_exclusion_policy.sh
2. scripts/ci/test_check_legacy_ingress_parser_drift.sh
3. scripts/ci/test_check_performance_thresholds.sh
4. scripts/ci/test_check_workflow_kolme_heavy_exclusion_policy.sh
5. scripts/ci/test_fallback_retirement_docs_parity_contract.sh
6. scripts/ci/test_generate_test_harness_loc_report.sh
7. scripts/ci/test_local_metrics_scrape_ci_exclusion_policy.sh
8. scripts/ci/test_local_retry_diagnostics_ci_exclusion_policy.sh
9. scripts/ci/test_run_kolme_test_harness_loc_soft_budget_contract_lane.sh
10. scripts/ci/test_run_with_retry.sh
11. scripts/ci/test_service_api_reason_code_compatibility_ci_exclusion_policy.sh
12. scripts/ci/test_service_api_serde_payload_parity_ci_exclusion_policy.sh
13. scripts/ci/test_service_api_validation_negative_matrix_ci_exclusion_policy.sh
14. scripts/ci/test_workflow_cache_policy.sh
15. scripts/ci/test_workflow_performance_policy.sh
16. scripts/runtime/test_run_input_mutation_coverage_guided_contract_lane.sh
17. scripts/runtime/test_run_input_mutation_coverage_guided_deep_lane.sh
18. scripts/runtime/test_run_live_network_smoke_contract_lane.sh
19. scripts/runtime/test_validate_async_runtime_live.sh
20. scripts/runtime/test_validate_libp2p_process_isolated_harness.sh

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Wave-1 migration inventory | Exactly 20 listed shell wrappers are removed from git-tracked source |
| C-02 | AC-2 | Functional | Rust migration suite execution | Rust suite passes and validates command/docs/policy markers for each migrated lane |
| C-03 | AC-2 | Regression | CI tool command surface + strategy docs | CI entrypoint and docs contracts reference the Rust migration suite and remain deterministic |
| C-04 | AC-3 | Conformance | Current shell test-file count, rust test-file count, baseline + thresholds + optional waiver | Policy resolves to `within|warn|fail` and fails closed for unwaived fail-level regression |

## Test Mapping
- C-01/C-02 -> `crates/kamn-core/tests/shell_test_surface_migration_wave1.rs`
- C-03 -> `scripts/ci/test_ci_tools_command_surface_contract.sh`, `scripts/ci/test_ci_strategy_contract.sh`, `crates/kamn-core/tests/ci_strategy_docs.rs`
- C-04 -> `crates/kamn-core/tests/shell_test_surface_ratio_policy.rs`

## Success Metrics
- Shell wrapper files removed in wave-1: `>= 20`.
- Rust replacement suite executes in CI via `scripts/ci/test_ci_tools.sh` fast/full paths.
- Shell-vs-rust test-file ratio policy gate is active and fail-closed with explicit waiver path.
