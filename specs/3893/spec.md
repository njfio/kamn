# Issue #3893 Spec

- Title: Subtask: add docs-contract and milestone-summary parity checks for activation closure
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Milestone closure must remain synchronized with documented activation markers and summary outputs.

## Scope
In:
- Add docs-contract and milestone-summary parity checks.

Out:
- Gate marker evaluation logic changes.

## Acceptance Criteria
- AC-1:  Docs or summary marker drift fails closed.
- AC-2:  Closure summary marker set remains deterministic.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Integration | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | docs-contract checker fails closed on tampered activation-closure marker docs with deterministic reason `activation_closure_docs_missing_marker:<marker>`. |
| C-02 | AC-2 | Unit/Functional | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | activation-closure summary marker key set is explicitly validated via deterministic docs-contract marker tuple. |
| C-03 | AC-3 | Regression | `cargo test -p kamn-core --test kolme_devnet_ops_docs` | existing plan/docs regression suite remains green with activation closure summary marker additions. |

## Test Mapping
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
