# Issue #5420 Plan — Daemon Tests Phase-3 Decomposition

## Approach
1. Extract inline runtime-focused daemon tests into `daemon_tests/runtime_contract_tests.rs`.
2. Extract inline live-postgres matrix tests (non-topology) into `daemon_tests/live_postgres_matrix_contract_tests.rs`.
3. Keep root `daemon_tests.rs` as a bounded shell with phase-3 marker and include routing for fixture/runtime/matrix/topology slices.
4. Update extraction and docs contract tests to assert phase-3 markers and updated root budget.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs` (new)
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/review/gaps-and-issues-r45.md`

## Risks / Mitigations
- Risk: selector drift from module extraction.
  - Mitigation: continue using `include!` inside `main_tests::daemon_tests`.
- Risk: docs marker drift.
  - Mitigation: update docs and fail-closed docs contract in same change.
- Risk: partial extraction leaves ambiguous ownership.
  - Mitigation: phase-3 marker names explicit runtime vs matrix ownership modules.

## Interfaces / Contracts
- Preserve existing test path prefix: `main_tests::daemon_tests::*`.
- Add phase-3 shell marker in `daemon_tests.rs` with required include declarations.
- Add ops-doc markers:
  - `daemon_tests_live_postgres_fixture_phase3_runtime_module_path=.../runtime_contract_tests.rs`
  - `daemon_tests_live_postgres_fixture_phase3_matrix_module_path=.../live_postgres_matrix_contract_tests.rs`
  - `daemon_tests_live_postgres_fixture_phase3_root_target_max_lines=<n>`

## Validation Strategy
- Red: tighten extraction/docs contract tests for phase-3 marker expectations.
- Green: move test blocks into new include modules and update docs markers.
- Verify: run targeted daemon/docs contracts, `cargo fmt --check`, and strict clippy.
