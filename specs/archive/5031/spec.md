# Issue #5031 Spec

- Title: Subtask: M2 DID auth + ABAC + RLS negative matrix and audit evidence fixtures
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5018` established baseline DID auth, ABAC authorization, RLS
templates, and audit hash-chain contracts. The remaining high-risk gap is an
explicit negative authorization matrix contract that emits deterministic audit
fixtures for denied cases and flags allow/deny drift.

## Acceptance Criteria
- AC-1: M2 ABAC reason markers are exported as public constants and used by
  runtime/tests instead of string literals.
- AC-2: M2 exposes deterministic negative-matrix evaluation API that returns
  `AllDenied`/`DriftDetected` with stable reason markers and per-case audit
  fixtures.
- AC-3: Negative-matrix evaluation fails closed for invalid case sets or invalid
  event timestamps.
- AC-4: Existing DID auth, RLS guard templates, and audit hash-chain integrity
  behavior remain deterministic and passing.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add negative-matrix contracts/API in `data_layer_m2_gateway_access`.
- Add stable ABAC reason-code constants and use them in code/tests.
- Add conformance tests for all-denied/drift-detected and invalid-matrix
  fail-closed behavior.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI/workflow/shell-surface modifications.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | Agent/owner/auditor/intruder ABAC decisions | Reason markers match exported constants |
| C-02 | AC-2 | Conformance | Negative matrix where all cases should deny | Decision is `AllDenied`, audit fixtures emitted deterministically |
| C-03 | AC-2 | Conformance | Matrix includes one unexpectedly allowed case | Decision is `DriftDetected` with mismatched case evidence |
| C-04 | AC-3 | Regression | Empty matrix or zero event timestamp | Fail-closed typed errors |
| C-05 | AC-4 | Regression | Existing DID auth, RLS, and audit hash-chain tests | Existing deterministic behavior remains green |
| C-06 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m2_gateway_access`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5031.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5031.json`

## Success Metrics
- Negative-matrix API produces deterministic drift evidence and audit fixtures.
- All M2 conformance cases pass in `data_layer_m2_gateway_access` suite.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
