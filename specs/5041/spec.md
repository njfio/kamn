# Issue #5041 Spec

- Title: Subtask: shell-neutral test orchestration guardrail and ratio-budget evidence policy
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5028` added PRD critical-scenario conformance, but there is no
single Rust-native policy contract that evaluates shell-neutral orchestration
evidence alongside shell/rust ratio-budget markers for deterministic release
gating.

PRD mapping:
- Section 18.2 conformance gating and deterministic evidence policy
- Shell-surface governance constraints from milestone execution plan
- Cross-cutting validation acceptance gates (M11 + governance)

## Acceptance Criteria
- AC-1: Policy contract deterministically evaluates shell-neutral orchestration
  compliance from critical-scenario evidence and shell/rust budget markers.
- AC-2: Policy blocks when orchestration evidence contains shell-mode
  violations, positive shell delta, or ratio fail-threshold breaches.
- AC-3: Policy warns (but does not block) when ratio exceeds warn threshold and
  remains below fail threshold.
- AC-4: Invalid threshold configuration fails closed with typed errors.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust policy module in `kamn-core` for shell-neutral orchestration and
  ratio-budget evidence evaluation.
- Conformance tests for verified/warn/blocked outcomes and fail-closed
  threshold validation.
- Root export wiring for policy API reuse.

Out of scope:
- New shell/python/workflow implementations.
- CI workflow graph changes.
- New dependencies or wire/protocol format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Conformant critical scenario report + non-increasing shell delta + ratio below warn | Policy decision is `Verified` with stable verified reason marker |
| C-02 | AC-2 | Conformance | Critical scenario report includes shell-mode violations | Policy decision is `Blocked` with orchestration-block reason marker |
| C-03 | AC-2 | Conformance | Positive shell delta or ratio above fail threshold | Policy decision is `Blocked` with deterministic budget-block reason markers |
| C-04 | AC-3 | Regression | Ratio above warn and below fail threshold | Policy decision is `Warning` with ratio-warn reason marker |
| C-05 | AC-4 | Regression | Invalid warn/fail threshold ordering | Fail-closed typed error |
| C-06 | AC-5 | Regression | Inspect diff paths and shell guardrails | No shell/workflow/python/template path changes; ratio/ceiling remain GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_shell_neutral_policy`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5041.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5041.json`

## Success Metrics
- Shell-neutral policy API is exported and tested via deterministic reason
  markers.
- All ACs map to passing `spec_c0x_*` policy tests.
- Shell-to-Rust ratio direction remains improved/neutral through Rust-only changes.
