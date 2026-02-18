# Issue #3883 Spec

- Title: Subtask: add policy checker and CI exclusion tests for native cutover lane
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Cutover policy must fail closed and remain out of fast CI to preserve cost and determinism.

## Scope
In:
- Add policy checker and CI-fast exclusion checks.

Out:
- Additional cutover scenario types.

## Acceptance Criteria
- AC-1:  Policy checker fails on missing or drifted cutover markers.
- AC-2:  CI selection excludes heavy cutover lane from fast gate.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional/Integration/Regression | `bash scripts/cutover/test_check_cutover_ci_exclusion_policy.sh` | Policy checker fails closed on missing contract lane marker, deep-lane leakage, missing ci-tools contract coverage, and strategy-doc marker drift with deterministic reason codes. |
| C-02 | AC-2 | Functional/Integration | `bash scripts/cutover/test_check_cutover_ci_exclusion_policy.sh` | CI boundary remains enforced: contract lane command exists in `ci-fast-gate`, deep lane command stays excluded from `ci-fast-gate` and `scripts/ci/test_ci_tools.sh`, and docs markers stay synchronized. |
| C-03 | AC-3 | Unit/Functional/Integration/Regression | `bash scripts/cutover/test_run_cutover_rollback_contract_lane.sh`; `cargo test -p kamn-core --test ci_strategy_docs doc_contains_cutover_ci_exclusion_policy_contract_markers -- --exact` | Contract-lane harness and docs contract assertions pass with cutover CI exclusion policy wiring present. |

## Test Mapping
- `scripts/cutover/test_check_cutover_ci_exclusion_policy.sh`
- `scripts/cutover/test_run_cutover_rollback_contract_lane.sh`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
