# Plan — #4195 Full-Stack Harness Marker Completeness Red Tests

## Approach
1. Extend the full I/O scenario matrix policy checker regression test script with tamper fixtures for:
- missing harness marker
- dry-run command-count parity mismatch
- dry-run command-status parity mismatch
2. Update `docs/ops/configuration.md` with full-stack harness mismatch controls and deterministic fail-closed reason markers.
3. Add docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.

## Affected Modules
- `scripts/runtime/test_check_full_io_scenario_matrix_live_policy.sh`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks / Mitigations
- Risk: brittle reason-code assertions if checker reason mapping changes.
- Mitigation: assert explicit deterministic reason markers that already exist in policy logic.

## Interfaces / Contracts
- Policy checker contract:
  - `full_io_scenario_matrix_policy_process_harness_mismatch`
  - `full_io_scenario_matrix_policy_dry_run_command_count_mismatch`
  - `full_io_scenario_matrix_policy_dry_run_command_status_mismatch`
