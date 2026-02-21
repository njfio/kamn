# Issue #5420 Spec — Daemon Tests Decomposition Phase 3

- Status: Reviewed
- Issue: #5420
- Parent: #3812
- Milestone: R27 Program: operational hardening and live validation

## Problem Statement
`crates/kamn-node/src/main_tests/daemon_tests.rs` remains a large mixed-surface test module after phase-2 extraction. Runtime shutdown tests and live-postgres matrix tests are still inline, which keeps review/debug friction high and invites monolith re-accumulation.

## Scope
In scope:
- Extract two additional coherent slices from `daemon_tests.rs` into `src/main_tests/daemon_tests/*.rs` include modules.
- Preserve `main_tests::daemon_tests::*` test path stability.
- Add phase-3 shell markers and docs markers for decomposition governance.

Out of scope:
- Runtime behavior changes.
- Renaming existing test functions.
- Multi-host distributed lane implementation.

## Acceptance Criteria
- AC-1: `daemon_tests.rs` routes runtime and matrix slices through include modules while retaining topology include routing from phase-2.
- AC-2: Decomposition contract tests fail closed when phase-3 include markers or bounded shell markers drift.
- AC-3: Ops/docs contracts include phase-3 module-path and line-budget markers.
- AC-4: Targeted daemon and docs contract suites pass with unchanged selector paths.

## Conformance Cases
- C-01 (Conformance, AC-2): extraction contract verifies phase-3 marker + required include set + bounded root line budget.
- C-02 (Functional, AC-1): runtime daemon shutdown contract test remains invokable under `main_tests::daemon_tests::*`.
- C-03 (Integration, AC-1/AC-4): live-postgres matrix integration test remains stable under existing selector path.
- C-04 (Regression, AC-3): docs contract enforces phase-3 decomposition marker keys and target budget values.

## Success Metrics
- `daemon_tests.rs` shrinks into a bounded shell with include routing for phase-1/2/3 slices.
- No selector/path regressions for existing daemon contract tests.
- Marker tests prevent inlining backslide.

## AC → Tests Mapping
- AC-1: `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant -- --exact`
- AC-2: `cargo test -p kamn-node --test main_module_extraction_contract main_module_extraction_contract_daemon_tests_decomposition_shell_markers_remain_stable -- --exact`
- AC-3: `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_daemon_tests_live_postgres_fixture_decomposition_markers -- --exact`
- AC-4: `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_graceful_shutdown_emits_structured_drain_markers -- --exact`
