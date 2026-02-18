# Issue #3891 Spec

- Title: Subtask: add activation readiness and budget marker checks to go-no-go policy
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Promotion gating must enforce both readiness markers and cost boundaries deterministically.

## Scope
In:
- Add activation readiness and budget marker checks.

Out:
- Docs parity checks.

## Acceptance Criteria
- AC-1:  Missing readiness markers fail closed.
- AC-2:  Budget threshold violations fail with deterministic reason codes.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `bash scripts/runtime/test_run_go_no_go_gate_lane.sh` | unit helper checks fail closed when `go_no_go_evidence_status` readiness marker is missing (`gate_required_artifact_status_mismatch:go_no_go_evidence`). |
| C-02 | AC-1 | Functional/Integration | `bash scripts/runtime/test_run_go_no_go_gate_lane.sh` | readiness-marker fault profile (`--fault-profile readiness_marker_missing`) fails closed with deterministic mismatch reason and `final_decision=NO-GO`. |
| C-03 | AC-2 | Functional/Integration | `bash scripts/runtime/test_run_go_no_go_gate_lane.sh` | budget fault profile (`--fault-profile runtime_budget_warn`) fails closed with `runtime_budget_exceeded`, `policy_outcome=FAIL`, and `final_decision=NO-GO`. |
| C-04 | AC-3 | Regression | `cargo test -p kamn-core --test ci_strategy_docs` | CI strategy docs remain synchronized with runtime go/no-go readiness and budget fail-closed marker contracts. |

## Test Mapping
- `scripts/runtime/test_run_go_no_go_gate_lane.sh`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
