# Issue #5028 Spec

- Title: Task: enforce PRD critical-scenario conformance matrix with shell-neutral test orchestration
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD Section 18.2 defines critical validation scenarios `62..71` as release
gates. The repository lacks one deterministic Rust contract that verifies all
required scenarios are present, passed, and executed through shell-neutral
orchestration policy (Rust-first by default).

PRD mapping:
- Section 18.2 critical scenarios (`62..71`)
- M11 cross-cutting validation and acceptance gates
- Shell-surface guardrail policy from milestone execution plan

## Acceptance Criteria
- AC-1: Contract surface deterministically tracks PRD critical scenarios
  `62..71` and validates matrix completeness.
- AC-2: Conformance evaluation deterministically reports `Conformant` only when
  all required scenarios pass.
- AC-3: Shell-neutral orchestration policy is enforced in conformance
  evaluation (Rust-only mode required by default for critical scenarios).
- AC-4: Invalid scenario IDs and duplicate/mutating records fail closed with
  stable reason markers.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust module in `kamn-core` for PRD critical scenario result recording,
  conformance evaluation, and shell-neutral orchestration enforcement.
- Conformance tests covering completeness, failure states, shell-policy
  violations, and fail-closed invalid inputs.
- Public API exports for downstream `#5041` integration and release evidence.

Out of scope:
- New shell/python/workflow orchestration scripts.
- Changes to CI workflow structure.
- New dependencies or wire/protocol format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Inspect required scenario catalog | Deterministic ordered list of scenario IDs `62..71` |
| C-02 | AC-2 | Conformance | Record all scenarios as passed | Conformance report returns `Conformant` with stable pass reason marker |
| C-03 | AC-2 | Regression | Record one scenario as failed | Conformance report returns `NonConformant` with failed-scenario reason marker |
| C-04 | AC-3 | Regression | Record scenario execution with shell-hybrid orchestration mode | Conformance report returns `NonConformant` with shell-neutral policy reason marker |
| C-05 | AC-4 | Regression | Record invalid scenario ID or mutate existing scenario record | Fail-closed typed error with stable reason marker |
| C-06 | AC-5 | Regression | Inspect issue diff paths and run shell guardrails | No shell/workflow/python/template path changes; ratio/ceiling remain GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_prd_critical_scenario_conformance`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5028.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5028.json`

## AC Verification
| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1 | ✅ | `spec_c01_required_scenario_catalog_is_deterministic` |
| AC-2 | ✅ | `spec_c02_all_required_scenarios_pass_with_rust_only_orchestration` |
| AC-3 | ✅ | `spec_c05_shell_hybrid_orchestration_is_policy_violation` |
| AC-4 | ✅ | `spec_c06_invalid_scenario_ids_and_mutating_records_fail_closed` |
| AC-5 | ✅ | `bash scripts/ci/check_shell_rust_ratio_guardrail.sh ...` and `bash scripts/ci/check_shell_loc_hard_ceiling.sh ...` with Rust-only diff |

## Success Metrics
- PRD critical scenario contract surface is exported through `kamn_core`.
- All ACs map to passing `spec_c0x_*` tests with deterministic reason markers.
- Shell-to-Rust ratio direction remains improved/neutral through Rust-only changes.
