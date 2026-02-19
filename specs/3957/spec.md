# Issue #3957 Spec

- Title: Subtask: create multi-signer quorum profile matrix fixtures and decision-path validation harness
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
Signer preflight enforces quorum contracts, but matrix coverage for allowed/disallowed profile combinations and deterministic decision-path outcomes is fragmented across single-scenario tests.

## Acceptance Criteria
- AC-1: Quorum profile matrix fixtures cover both pass and fail decision paths for single-signer and failover signer profiles.
- AC-2: Decision-path harness validates deterministic fail-closed reason markers for quorum shortfall, linkage violation, and failover previous-profile omission.
- AC-3: Functional and integration tests exercise matrix fixtures through existing signer preflight paths without introducing parallel policy logic.
- AC-4: `docs/ops/configuration.md` declares quorum matrix fixture profile contracts and validation commands.

## Scope
In scope:
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/3957/spec.md`
- `specs/3957/plan.md`
- `specs/3957/tasks.md`

Out of scope:
- New shell/python/workflow lanes.
- Quorum drift policy checker go/no-go lane implementation (handled by #3958).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | single-signer profile with default approvals | readiness succeeds and `quorum_linked=true` |
| C-02 | AC-1/AC-2 | Functional | non-failover profile missing from approved signer set | fail-closed with `runtime_signer_quorum_linkage_violation` |
| C-03 | AC-1/AC-2 | Functional | non-failover approvals shortfall | fail-closed with `runtime_signer_attestation_quorum_shortfall` |
| C-04 | AC-1/AC-2 | Functional | failover profile with previous signer omitted from approved set | fail-closed with `runtime_signer_failover_attestation_previous_profile_not_approved` |
| C-05 | AC-3 | Integration | signer preflight matrix run through main test harness | mixed pass/fail cases preserve deterministic markers |
| C-06 | AC-4 | Functional | ops configuration docs contract markers | docs assert quorum matrix fixture contract + validation commands |

## Test Mapping
- `cargo test -p kamn-node signer::tests::functional_signer_preflight_quorum_decision_path_matrix -- --exact --nocapture`
- `cargo test -p kamn-node main_tests::signer_tests::integration_kolme_live_signer_preflight_quorum_profile_matrix_paths -- --exact --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_signer_quorum_profile_matrix_controls -- --exact`

## Success Metrics
- Matrix harness covers deterministic pass/fail profile combinations without policy ambiguity.
- Signer preflight reason markers remain stable under matrix evaluation.
- Shell LOC delta remains `0`.
