# Issue #3960 Spec

- Title: Subtask: add docs-contract and config-layer parity tests for signer provenance and fallback prohibition
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
Signer provenance/fallback policy markers exist across docs and runtime code, but there is no dedicated contract test that fails closed when docs marker taxonomy drifts from config/runtime policy surfaces.

## Acceptance Criteria
- AC-1: Docs-contract tests fail closed when signer fallback-prohibition and provenance markers drift in CI/docs surfaces.
- AC-2: Config-layer parity test verifies runtime signer key-source policy reason-code CSV markers remain aligned with source reason-code taxonomy.
- AC-3: Deterministic remediation guidance and guard commands are documented for the new contract lane.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Scope
In scope:
- `crates/kamn-node/tests/signer_provenance_fallback_policy_contract.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `specs/3960/spec.md`
- `specs/3960/plan.md`
- `specs/3960/tasks.md`

Out of scope:
- Runtime signer execution behavior changes.
- Deployment shell workflow changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | docs contract markers in `docs/ci/strategy.md` and `docs/ops/configuration.md` | missing marker fails closed |
| C-02 | AC-2 | Integration | source + docs reason-code CSV parity check | source reason markers and docs CSV remain aligned |
| C-03 | AC-3 | Unit | marker extraction/parsing helper | invalid/missing marker extraction fails deterministically |
| C-04 | AC-4 | Regression | dedicated signer provenance/fallback policy contract test | drift regression remains fail closed |

## Test Mapping
- `cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture`
- `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_signer_material_validation_and_fallback_prohibition_contracts -- --exact --nocapture`

## Success Metrics
- Signer provenance and fallback policy marker drift fails in Rust docs-contract/config parity tests.
- No shell LOC increase; governance stays within existing shell/rust ratio constraints.
