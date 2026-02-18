# Issue #3644 Spec

- Title: `Task: ship TLS go-no-go validation lanes and operational runbooks`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
TLS work required deterministic go/no-go evidence and operational runbook checkpoint contracts to make rollout and rollback auditable.

## Scope
In:
- TLS go/no-go evidence generation and contract-lane validation.
- Runbook/checklist contract coverage for rollout and rollback checkpoints.
- Deterministic reason taxonomy for TLS evidence.

Out:
- Provider-specific certificate provisioning automation.

## Acceptance Criteria
- AC-1: TLS go/no-go lane emits deterministic evidence markers.
- AC-2: release/runbook docs contracts include required TLS checkpoints.
- AC-3: policy checks fail closed for missing or invalid TLS evidence markers.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | TLS evidence bundle generation passes |
| C-02 | AC-1/AC-3 | Conformance | `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh` | go/no-go contract lane enforces TLS markers |
| C-03 | AC-2 | Regression | `cargo test -p kamn-core --test release_gonogo_checklist_docs` | checklist docs contract passes |
| C-04 | AC-2 | Regression | `cargo test -p kamn-core --test tls_feature_gate_ci_docs` | TLS CI docs contract passes |
| C-05 | AC-2/AC-3 | Regression | `cargo test -p kamn-core --test tls_dependency_governance_docs` | dependency governance docs contract passes |

## Test Mapping
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/tls_feature_gate_ci_docs.rs`
- `crates/kamn-core/tests/tls_dependency_governance_docs.rs`

## Success Metrics
- TLS go/no-go lane is deterministic and fail closed.
- Runbook/checklist contracts stay synchronized.
