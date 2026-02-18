# Issue #3806 Spec

- Title: `Subtask: extend TLS rollout/rollback runbook checkpoint contracts`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Operational rollout/rollback safety for TLS needed explicit checkpoint markers in runbooks and deterministic docs-contract validation.

## Scope
In:
- Extend TLS runbook/checklist checkpoint matrix contracts.
- Enforce required checkpoint marker presence via tests.
- Keep promotion evidence fail-closed when checkpoint markers drift.

Out:
- Infrastructure-specific runbook variants.

## Acceptance Criteria
- AC-1: rollout/rollback checkpoint matrix is represented in docs contracts.
- AC-2: docs checks fail closed on missing checkpoint markers.
- AC-3: go/no-go lane remains synchronized with runbook checkpoint requirements.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Regression | `cargo test -p kamn-core --test release_gonogo_checklist_docs` | release checklist checkpoint markers are enforced |
| C-02 | AC-1/AC-2 | Regression | `cargo test -p kamn-core --test tls_feature_gate_ci_docs` | TLS CI feature-gate docs markers are enforced |
| C-03 | AC-1/AC-3 | Regression | `cargo test -p kamn-core --test tls_dependency_governance_docs` | TLS governance docs checkpoint markers are enforced |
| C-04 | AC-3 | Conformance | `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh` | go/no-go evidence lane remains aligned with docs contracts |

## Test Mapping
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/tls_feature_gate_ci_docs.rs`
- `crates/kamn-core/tests/tls_dependency_governance_docs.rs`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`

## Success Metrics
- TLS rollout/rollback checkpoint markers are deterministic and contract-enforced.
