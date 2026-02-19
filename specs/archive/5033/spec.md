# Issue #5033 Spec

- Title: Subtask: M4 escrow message visibility and settlement evidence integrity contracts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5020` established baseline M4 escrow transitions, scoped
visibility, and settlement evidence hash chains. The remaining high-risk gap is
explicit deterministic linkage between terminal escrow projection and latest
settlement evidence record, plus exported reason-marker constants for transition
and visibility decisions.

## Acceptance Criteria
- AC-1: M4 transition and visibility reason markers are exported as public
  constants and used by runtime code/tests instead of string literals.
- AC-2: M4 exposes deterministic settlement-evidence reconciliation contracts
  that compare terminal escrow state/receipt against latest evidence row and
  return `Match`/`Mismatch` decision with stable reason markers.
- AC-3: Reconciliation fails closed for non-terminal escrow projections and
  detects missing/mismatched settlement evidence deterministically.
- AC-4: Existing visibility and evidence hash-chain integrity behavior remains
  deterministic and passing.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Export M4 reason-marker constants and align tests with constants.
- Add settlement-evidence reconciliation API/types in
  `data_layer_m4_escrow_integration`.
- Add conformance tests for match/mismatch/non-terminal fail-closed paths.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI/workflow/shell-surface modifications.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | Transition/visibility paths in M4 | Reason codes use exported constants |
| C-02 | AC-2 | Conformance | Terminal escrow + matching latest evidence row | Reconciliation report decision is `Match` |
| C-03 | AC-2 | Conformance | Terminal escrow + mismatched or missing evidence | Reconciliation report decision is `Mismatch` |
| C-04 | AC-3 | Regression | Non-terminal escrow reconciliation request | Fail-closed typed error |
| C-05 | AC-4 | Regression | Existing visibility + hash-chain tamper tests | Existing deterministic assertions remain green |
| C-06 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m4_escrow_integration`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5033.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5033.json`

## Success Metrics
- Reconciliation reports provide deterministic `Match`/`Mismatch` evidence for
  M4 settlement linkage.
- All M4 conformance cases pass in `data_layer_m4_escrow_integration` suite.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
