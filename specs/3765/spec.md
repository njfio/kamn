# Issue #3765 Spec

- Title: `Subtask: wire TLS go-no-go lane into release gate with deterministic reason taxonomy`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
TLS deployment safety required deterministic promotion evidence in release go/no-go surfaces with fail-closed reason taxonomy.

## Scope
In:
- Wire TLS lane outputs into release go/no-go evidence bundle.
- Enforce deterministic reason taxonomy for marker validation.
- Add regression checks for marker drift.

Out:
- Certificate lifecycle automation.

## Acceptance Criteria
- AC-1: go/no-go evidence includes TLS marker outputs.
- AC-2: policy checks fail closed on TLS marker drift.
- AC-3: docs governance remains synchronized with TLS evidence contract.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | evidence bundle emits TLS markers |
| C-02 | AC-1/AC-2 | Conformance | `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh` | contract lane validates TLS marker taxonomy |
| C-03 | AC-2 | Regression | `cargo test -p kamn-core --test release_gonogo_checklist_docs` | checklist contract includes TLS go/no-go markers |
| C-04 | AC-3 | Regression | `cargo test -p kamn-core --test tls_dependency_governance_docs` | docs governance contract remains synchronized |

## Test Mapping
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/tls_dependency_governance_docs.rs`

## Success Metrics
- TLS go/no-go evidence and reason taxonomy checks are deterministic and fail closed.
