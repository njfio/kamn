# Issue #3915 Spec

- Title: `Task: enforce signer secret-lifecycle policy and docs parity contracts`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer secret-lifecycle policy must fail closed when fallback key violations or marker drift occur, and docs must stay synchronized with policy expectations.

## Scope
In:
- Add policy-check coverage for fallback-secret prohibition and required lifecycle markers.
- Add docs-contract parity checks for signer secret-lifecycle marker sets.
- Update CI strategy and production next-steps docs with closure markers.

Out:
- Runtime transport changes.
- External key custody redesign.

## Acceptance Criteria
- AC-1: Given policy checks, when fallback-secret reason appears, then policy fails closed deterministically.
- AC-2: Given lifecycle marker checks, when required markers are missing, then policy fails closed deterministically.
- AC-3: Given docs parity contracts, when required marker docs drift, then tests fail closed.
- AC-4: Given scoped signer suites, when regressions run, then signer behavior remains stable.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_rejects_fallback_secret_violation_reason_code -- --exact --nocapture` | fallback-secret reason code is rejected |
| C-02 | AC-2 | Functional/Conformance | `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_rejects_missing_required_lifecycle_markers -- --exact --nocapture` | missing required marker set is rejected |
| C-03 | AC-3 | Docs/Conformance | `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract docs_declare_signer_secret_lifecycle_policy_markers_and_closure_chain -- --exact --nocapture` | CI and plan docs include required policy/closure markers |
| C-04 | AC-4 | Integration/Regression | `cargo test -p kamn-node signer -- --nocapture` + `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | signer behavior remains parity-stable |

## Test Mapping
- `crates/kamn-node/tests/signer_secret_lifecycle_policy_contract.rs`
- scoped signer suites in `crates/kamn-node/src/main_tests/signer_tests.rs` and `crates/kamn-node/src/signer.rs`

## Success Metrics
- signer secret-lifecycle policy and docs parity contracts fail closed on drift.
