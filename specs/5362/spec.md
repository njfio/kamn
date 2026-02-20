# Issue #5362 Spec

- Title: Task: codify parallel lane fingerprint-schema contracts for live-postgres daemon validation
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5360` added multi-permutation invariance for sorted lane fingerprints, but fingerprint schema/field-order semantics are implicit in implementation. Format drift could weaken conformance guarantees without explicit schema contracts.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical fingerprint schema version + field-order contracts for parallel lane projections.
- AC-2: integration tests assert repeated-run fingerprint projections remain schema-conformant and deterministic for bounded same-host lane sets.
- AC-3: `docs/ops/configuration.md` includes explicit `#5362` fingerprint schema markers and validation commands.
- AC-4: docs-contract tests fail closed on fingerprint schema marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only fingerprint schema contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- Multi-host network topology orchestration.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | fingerprint schema helper contracts | schema version, field-order CSV, and formatted fingerprint structure remain canonical |
| C-02 | AC-2 | Integration | repeated fingerprint projection runs | identical sorted fingerprints; each fingerprint has valid field count/order and stable taxonomy/version fields |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | fingerprint schema markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_fingerprint_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_fingerprint_schema_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Parallel lane fingerprint schema drift becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on fingerprint schema marker regressions.
- R45 next-frontier narrative reflects fingerprint-schema hardening.
