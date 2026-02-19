# Issue #3959 Spec

- Title: Subtask: enforce production fallback-key denylist and fail-closed signer provenance taxonomy
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
`enforce_kolme_live_signer_key_source_policy` rejects strict `env-local` key-source use but does not classify fallback signer-secret env presence at the same policy boundary. Fallback rejection currently occurs later in signer preflight, leaving the early runtime key-source policy taxonomy incomplete.

## Acceptance Criteria
- AC-1: Production-targeted signer key-source policy rejects fallback signer secret env presence with deterministic fail-closed reason code `fallback_signer_secret_present_violation`.
- AC-2: Runtime signer key-source policy taxonomy remains deterministic and includes both `production_signer_key_source_env_local_forbidden` and `fallback_signer_secret_present_violation`.
- AC-3: Unit, Functional, Integration, and Regression tests map to the above behavior and pass.

## Scope
In scope:
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/arg_and_signer_policy_tests.rs`
- `specs/3959/spec.md`
- `specs/3959/plan.md`
- `specs/3959/tasks.md`

Out of scope:
- Managed signer backend command execution changes.
- Deployment shell/workflow changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | strict signer policy call with fallback env set | error contains `fallback_signer_secret_present_violation` |
| C-02 | AC-2 | Unit | runtime signer taxonomy helper/constant | taxonomy includes both production env-local and fallback reason codes |
| C-03 | AC-3 | Integration | strict managed-external key-source policy path with no fallback env | policy passes |
| C-04 | AC-3 | Regression | existing strict env-local rejection path | reason code remains `production_signer_key_source_env_local_forbidden` |

## Test Mapping
- `cargo test -p kamn-node unit_kolme_live_signer_key_source_policy_classifier_matrix -- --exact --nocapture`
- `cargo test -p kamn-node regression_kolme_live_signer_key_source_policy_rejects_fallback_secret_path_with_deterministic_reason_code -- --exact --nocapture`
- `cargo test -p kamn-node functional_kolme_live_strict_env_local_key_source_rejects_with_reason_code -- --exact --nocapture`
- `cargo test -p kamn-node integration_kolme_live_strict_managed_external_key_source_policy_passes -- --exact --nocapture`

## Success Metrics
- Fallback key-path denial is enforced at runtime key-source policy gate with deterministic reason taxonomy.
- Targeted signer policy test suite remains green without increasing shell LOC surface.
