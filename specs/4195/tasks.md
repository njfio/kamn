# Tasks — #4195 Full-Stack Harness Marker Completeness Red Tests

## Ordered Tasks
- T1 (Regression): add missing-marker and dry-run parity mismatch tamper tests in `scripts/runtime/test_check_full_io_scenario_matrix_live_policy.sh`.
- T2 (Docs): document full-stack harness mismatch controls in `docs/ops/configuration.md`.
- T3 (Docs Contract): add docs assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- T4 (Verify): run targeted shell and docs-contract tests.

## Test Tier Mapping
- Unit: N/A (shell/docs contract scope)
- Functional: full I/O policy checker happy path
- Integration: policy checker composition in shell test script
- Regression: tampered missing-marker and parity-mismatch fixtures
- Performance: N/A (bounded CI smoke path only)
