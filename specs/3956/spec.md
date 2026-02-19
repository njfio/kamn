# Issue #3956 Spec

- Title: Subtask: add rotation freshness enforcement and stale-key rejection regression coverage
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
Signer preflight enforces stale rotation epochs when failover is active, but rotation metadata regression in non-failover flows is not typed as an explicit freshness outcome and is not fail-closed with dedicated regression coverage.

## Acceptance Criteria
- AC-1: Rotation freshness evaluation uses typed outcomes and deterministic reason codes.
- AC-2: Stale/invalid rotation metadata is rejected in both failover and non-failover stale-key paths.
- AC-3: Unit, Functional, Integration, and Regression tests are present and passing for stale rotation enforcement.
- AC-4: `docs/foundation/runtime-watchdog-attestation.md` declares rotation freshness reason markers.

## Scope
In scope:
- `crates/kamn-node/src/signer/signer_policy.rs`
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- `docs/foundation/runtime-network.md`
- `docs/foundation/runtime-watchdog-attestation.md`
- `crates/kamn-core/tests/runtime_watchdog_attestation_docs.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/3956/spec.md`
- `specs/3956/plan.md`
- `specs/3956/tasks.md`

Out of scope:
- External rotation orchestration automation.
- New shell/python/workflow lanes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | rotation freshness helper matrix | typed outcome is deterministic for fresh/failover-stale/non-failover-regressed |
| C-02 | AC-2 | Functional | preflight with non-failover regressed epoch metadata | fail-closed with deterministic stale-key reason |
| C-03 | AC-2/AC-3 | Integration | signer preflight via main test harness with stale metadata | integration path rejects stale metadata deterministically |
| C-04 | AC-3 | Regression | signer reason taxonomy + stale failover/non-failover guards | reason-code drift and stale-path regressions fail closed |
| C-05 | AC-4 | Functional | watchdog/docs marker assertions | runtime-watchdog attestation doc includes rotation freshness markers and guard commands |

## Test Mapping
- `cargo test -p kamn-node signer::signer_policy::tests::unit_signer_rotation_freshness_outcome_matrix -- --exact --nocapture`
- `cargo test -p kamn-node signer::tests::regression_signer_preflight_rejects_non_failover_rotation_epoch_regression -- --exact --nocapture`
- `cargo test -p kamn-node main_tests::signer_tests::integration_kolme_live_signer_preflight_rejects_non_failover_rotation_regression -- --exact --nocapture`
- `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract -- --nocapture`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs -- --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_multi_signer_quorum_signature_decision_controls -- --exact`

## Success Metrics
- Stale rotation metadata is fail-closed for both failover and non-failover paths.
- Deterministic rotation-freshness reason taxonomy is documented and contract-tested.
- Shell LOC delta remains `0`.
