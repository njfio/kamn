# Issue #4477 Spec

- Title: `Task: enforce tls evidence bundle completeness-freshness convergence in release gate checks`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-41-tls-governance-completion-and-anti-flake-merge-gate-reliability-contracts/index.md`
- Parent: `#4474`

## Problem Statement
Release go/no-go checks need deterministic TLS evidence completeness and freshness enforcement so missing/stale TLS evidence fails closed with auditable reason taxonomy.

## Scope
In:
- Go/no-go evidence contract support for TLS evidence convergence checks.
- Deterministic reason taxonomy for missing/stale/invalid TLS evidence paths.
- Release checklist docs markers and docs-contract parity checks.

Out:
- External evidence storage redesign.
- New TLS dependency posture checker implementation.

## Acceptance Criteria
- AC-1: missing or stale TLS evidence fails closed in go/no-go bundle decisioning.
- AC-2: TLS evidence convergence reason outputs are deterministic and auditable.
- AC-3: integration contract tests validate TLS evidence gate convergence and tamper rejection.
- AC-4: release checklist docs include TLS evidence completeness-freshness gate markers, enforced by docs tests.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | missing/stale TLS evidence yields deterministic NO-GO |
| C-02 | AC-2 | Unit | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | deterministic TLS evidence reason taxonomy markers are emitted |
| C-03 | AC-3 | Integration | `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh` | contract lane validates convergence path and deterministic decision surface |
| C-04 | AC-3 | Regression | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | tampered TLS evidence gate payload is rejected fail-closed |
| C-05 | AC-2 | Performance | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | bounded go/no-go checker path remains lightweight |
| C-06 | AC-4 | Docs | `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_tls_evidence_completeness_freshness_gate -- --exact` | release checklist TLS evidence gate markers remain present |

## Test Mapping
- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Success Metrics
- TLS evidence convergence checks produce deterministic GO/NO-GO decisions.
- Missing/stale/tampered TLS evidence is fail-closed with stable reason codes.
- Checklist docs marker drift is blocked by docs-contract tests.
