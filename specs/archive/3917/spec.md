# Issue #3917 Spec

- Title: `Subtask: add docs-contract parity checks for signer secret-lifecycle markers`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer secret-lifecycle docs can drift from policy-check expectations without explicit docs-contract tests.

## Scope
In:
- Add docs-contract checks for required lifecycle marker and forbidden reason-code declarations.
- Validate closure chain markers in production next-steps planning doc.

Out:
- Policy rule implementation changes.

## Acceptance Criteria
- AC-1: CI strategy docs include required signer secret-lifecycle policy markers and guard command.
- AC-2: Production next-steps doc includes signer secret-hardening closure chain/marker criteria.
- AC-3: Missing docs markers fail closed via contract tests.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Docs/Conformance | `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract docs_declare_signer_secret_lifecycle_policy_markers_and_closure_chain -- --exact --nocapture` | docs marker parity remains enforced |
| C-02 | AC-2 | Docs/Conformance | same command | closure chain markers remain present in next-steps doc |
| C-03 | AC-3 | Regression | remove/rename marker in docs | contract test fails closed |

## Test Mapping
- `crates/kamn-node/tests/signer_secret_lifecycle_policy_contract.rs`

## Success Metrics
- docs marker parity is deterministic and fail-closed.
