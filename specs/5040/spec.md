# Issue #5040 Spec

- Title: Subtask: M11 security-chaos-performance closure evidence and acceptance report
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5027` delivered M11 hardening readiness contracts and `#5028`
delivered PRD critical-scenario conformance contracts, but there is no
deterministic acceptance report that composes these artifacts into one
release-closure decision for operator signoff.

PRD mapping:
- Section 18 closure criteria (security + chaos + performance gates)
- M11 hardening acceptance requirements
- Operator readiness acceptance/reporting dependency

## Acceptance Criteria
- AC-1: M11 closure report contract deterministically composes hardening
  readiness and PRD critical-scenario conformance evidence into an acceptance
  decision.
- AC-2: Acceptance report rejects closure when performance/signoff evidence is
  missing, with stable reason markers.
- AC-3: Input validation fails closed for empty release marker values.
- AC-4: New closure report API is exported from `kamn_core`.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust closure-report module in `kamn-core`.
- Conformance tests for accept/reject paths and deterministic reason markers.
- Root export wiring for downstream runbook/ops report surfaces.

Out of scope:
- New shell/python/workflow orchestration.
- New dependencies or protocol/wire-format changes.
- CI workflow graph changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Submit conformant hardening+critical evidence and complete closure gates | Acceptance report returns `Accepted` with stable accept reason marker |
| C-02 | AC-1/AC-2 | Conformance | Submit `NoGo` hardening evidence | Acceptance report returns `Rejected` with hardening-block reason marker |
| C-03 | AC-1/AC-2 | Conformance | Submit non-conformant critical-scenario evidence | Acceptance report returns `Rejected` with critical-scenario-block reason marker |
| C-04 | AC-2 | Regression | Submit incomplete performance/signoff evidence | Acceptance report returns `Rejected` with stable evidence-gap reason markers |
| C-05 | AC-3 | Regression | Submit empty release marker | Fail-closed typed error |
| C-06 | AC-5 | Regression | Inspect diff paths and shell guardrails | No shell/workflow/python/template path changes; ratio/ceiling remain GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m11_closure_evidence`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5040.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5040.json`

## AC Verification
| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1 | ✅ | `spec_c01_conformant_closure_evidence_is_accepted`, `spec_c02_hardening_nogo_blocks_closure_acceptance`, `spec_c03_non_conformant_critical_scenario_blocks_closure_acceptance` |
| AC-2 | ✅ | `spec_c04_missing_performance_or_signoff_evidence_blocks_closure_acceptance` |
| AC-3 | ✅ | `spec_c05_empty_release_marker_fails_closed` |
| AC-4 | ✅ | `cargo test -p kamn-core --test data_layer_m11_closure_evidence` imports closure API from `kamn_core` root |
| AC-5 | ✅ | `bash scripts/ci/check_shell_rust_ratio_guardrail.sh ...` and `bash scripts/ci/check_shell_loc_hard_ceiling.sh ...` with Rust-only diff |

## Success Metrics
- Closure acceptance API is exported and consumed from `kamn_core`.
- All ACs map to passing `spec_c0x_*` tests with deterministic reason markers.
- Shell-to-Rust ratio direction remains improved/neutral through Rust-only changes.
