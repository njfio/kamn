# Issue #3914 Spec

- Title: `Subtask: add regression checks for signer secret redaction and decode-failure hygiene`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Without dedicated regression checks, decode failures can regress and leak raw private-key values in diagnostics.

## Scope
In:
- Add regression test ensuring decode failures do not include raw key input.
- Add docs policy notes and contract checks for secret-redaction hygiene.

Out:
- CI lane orchestration changes.

## Acceptance Criteria
- AC-1: Decode-failure regression test verifies sensitive input is redacted from error surfaces.
- AC-2: Source/docs contract checks guard signer secret-hygiene markers.
- AC-3: Scoped signer suites remain green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | `cargo test -p kamn-node signer::tests::regression_signer_private_key_decode_failure_redacts_sensitive_input -- --exact --nocapture` | decode failure message excludes raw key input |
| C-02 | AC-2 | Conformance/Docs | `cargo test -p kamn-node --test signer_secret_hygiene_contract -- --nocapture` | source/docs markers remain present |
| C-03 | AC-3 | Integration/Regression | `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | signer path parity remains stable |

## Test Mapping
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/tests/signer_secret_hygiene_contract.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`

## Success Metrics
- decode-failure redaction regression is explicit and stable.
- secret-hygiene docs/source parity is fail-closed.
