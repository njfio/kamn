# Testing Structure Contracts

This document defines deterministic decomposition and structural-budget guardrails for
the node test surface.

## Main Tests Decomposition Drift Cases (Issue #4452)

Issue lineage:
- Story: `#4444`
- Task: `#4447`
- Subtask: `#4452`

Deterministic decomposition taxonomy markers:
- `main_tests_decomposition_reason_taxonomy_version=kamn.testing.main-tests-decomposition-reason-taxonomy.v1`
- `main_tests_decomposition_reason_codes_csv=main_tests_domain_module_missing,main_tests_inline_monolith_reintroduced,main_tests_structural_budget_boundary_exceeded`

Deterministic decomposition and budget status markers:
- `main_tests_decomposition_status=verified`
- `main_tests_structural_budget_status=verified`

Guard commands:
- `cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test docs_contract_matrix_wave2_harness -- --nocapture`
- `bash scripts/ci/test_check_test_harness_loc_soft_budget.sh`
- `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
