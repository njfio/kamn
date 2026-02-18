# Issue #3807 Spec

- Title: `Subtask: add signer_policy reason-taxonomy drift and docs parity contracts`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer-policy reason markers must stay deterministic across source and docs. Drift (rename/remove/missing docs coverage) weakens operator remediation and regression confidence.

## Scope
In:
- Add explicit signer-policy reason-taxonomy contract checks that fail closed on drift.
- Add docs parity checks for `docs/foundation/runtime-network.md`.
- Keep runtime signing behavior unchanged.

Out:
- Any signer policy logic redesign.
- Runtime protocol or wire-format changes.

## Acceptance Criteria
- AC-1: Given signer policy source contracts, when taxonomy contract tests run, then required signer-policy reason markers remain present.
- AC-2: Given runtime-network docs contracts, when docs parity tests run, then signer policy reason taxonomy markers remain documented.
- AC-3: Given scoped kamn-node verification, when signer/doc tests run, then all targeted tests pass without behavioral drift.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Conformance | `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract source_declares_required_signer_policy_reason_markers -- --exact --nocapture` | signer-policy source contains required reason markers |
| C-02 | AC-2 | Functional/Conformance | `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract docs_runtime_network_declares_signer_policy_reason_taxonomy_markers -- --exact --nocapture` | runtime-network docs contain required signer-policy taxonomy markers |
| C-03 | AC-3 | Integration/Regression | `cargo test -p kamn-node signer -- --nocapture` + `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | signer behavior and taxonomy checks remain green |

## Test Mapping
- C-01: `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- C-02: `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- C-03: `crates/kamn-node/src/main_tests/signer_tests.rs` and signer module unit tests

## Success Metrics
- Required signer-policy reason markers are guarded by deterministic tests.
- `docs/foundation/runtime-network.md` documents signer-policy taxonomy coverage.
- Scoped signer suites pass unchanged.
