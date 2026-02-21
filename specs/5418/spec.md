# Issue #5418 Spec — Daemon Tests Decomposition Phase 2

- Status: Reviewed
- Issue: #5418
- Parent: #3812
- Milestone: R27 Program: operational hardening and live validation

## Problem Statement
`crates/kamn-node/src/main_tests/daemon_tests.rs` is still a large monolith (~4.1K lines) after phase-1 fixture extraction. The dense live-postgres topology contract slice increases review cost and makes incremental changes riskier.

## Scope
In scope:
- Extract a coherent live-postgres topology contract test block into `src/main_tests/daemon_tests/` include module(s).
- Preserve existing `main_tests::daemon_tests::*` test path names and behavior.
- Add decomposition guard assertions and update docs markers for phase-2 extraction.

Out of scope:
- Runtime behavior changes.
- Test renaming or command-path changes in operator docs.

## Acceptance Criteria
- AC-1: `daemon_tests.rs` line count is reduced by moving topology contract tests into submodule include file(s) while preserving function names.
- AC-2: Decomposition guard tests fail closed if inline topology test bodies are reintroduced or include markers disappear.
- AC-3: Docs marker contracts reflect phase-2 decomposition module path + bounded shell target.
- AC-4: Targeted daemon/live-postgres/doc contract tests pass unchanged.

## Conformance Cases
- C-01 (Unit, AC-2): extraction contract test verifies `daemon_tests.rs` contains decomposition marker + include entries and stays within phase-2 line budget.
- C-02 (Functional, AC-1): canonical topology contract test path still executes under `main_tests::daemon_tests::*` naming.
- C-03 (Integration, AC-4): topology digest stability integration test remains green after extraction.
- C-04 (Regression, AC-3): docs-contract test enforces phase-2 marker strings and command path contract stability.

## Success Metrics
- `daemon_tests.rs` drops below the phase-2 target bound.
- No behavior/test-path regressions in live-postgres topology contract tests.
- Docs + extraction contract checks prevent monolith re-accumulation.

## AC → Tests Mapping
- AC-1: `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_contract_is_canonical -- --exact`
- AC-2: `cargo test -p kamn-node --test main_module_extraction_contract main_module_extraction_contract_daemon_tests_decomposition_shell_markers_remain_stable -- --exact`
- AC-3: `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_daemon_tests_live_postgres_fixture_decomposition_markers -- --exact`
- AC-4: `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable -- --exact`
