# Issue #5027 Spec

- Title: Task: M11 execute hardening matrix (security, chaos, perf) and operator readiness
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M11 requires deterministic hardening closure evidence across security,
chaos, performance, and operator readiness checks. The codebase has many
independent tests and docs but lacks a unified Rust contract that composes
scenario outcomes into a fail-closed operator readiness decision with stable
reason markers.

PRD mapping:
- Section 15 (security architecture and audit controls)
- Section 17 (performance constraints and observability closure)
- Section 18 (test strategy including chaos and resilience scenarios)
- Milestone table M11 deliverables (security matrix + chaos + perf + ops closure)

## Acceptance Criteria
- AC-1: Hardening matrix contract deterministically tracks required scenario
  outcomes across security, chaos, performance, and operator-readiness domains.
- AC-2: Operator readiness evaluation returns deterministic GO/NO-GO output and
  stable reason markers based on scenario pass/fail and severity.
- AC-3: Fail-closed behavior rejects duplicate scenario IDs, missing required
  scenario results, and invalid severity/status transitions.
- AC-4: Public M11 API is exported from `kamn_core` for downstream report and
  runbook integration.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M11 module in `kamn-core` for hardening scenario registration,
  execution outcome recording, and operator readiness decision synthesis.
- Conformance tests for deterministic ordering, fail-closed guards, and
  readiness reason-marker stability.
- Public API exports for downstream subtask `#5040` closure evidence reporting.

Out of scope:
- New shell/python/workflow orchestration.
- External benchmark harness changes.
- New dependencies or wire/protocol format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Register required scenarios across all M11 domains | Deterministic scenario catalog projection and ordering |
| C-02 | AC-2 | Conformance | Record passing outcomes for all required scenarios | Readiness evaluates `Go` with stable positive reason marker |
| C-03 | AC-2/AC-3 | Regression | Record at least one critical failure outcome | Readiness evaluates `NoGo` with deterministic blocking reason marker |
| C-04 | AC-3 | Regression | Register duplicate scenario IDs | Fail-closed typed error with stable reason marker |
| C-05 | AC-3 | Regression | Evaluate readiness with missing required outcomes | Fail-closed `NoGo`/error path with deterministic missing-scenario marker |
| C-06 | AC-4 | Conformance | Import M11 API from `kamn_core` root export | Module symbols available without direct module-path fallback |
| C-07 | AC-5 | Regression | Inspect issue diff paths and run shell guardrails | No shell/workflow/python/template path changes; ratio/ceiling remain GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m11_hardening_readiness`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5027.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5027.json`

## AC Verification
| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1 | ✅ | `spec_c01_registration_catalog_is_deterministic_and_sorted` |
| AC-2 | ✅ | `spec_c02_all_required_pass_results_in_operator_readiness_go`, `spec_c03_critical_failure_for_required_scenario_blocks_readiness` |
| AC-3 | ✅ | `spec_c04_duplicate_scenario_registration_fails_closed`, `spec_c05_missing_required_scenario_outcomes_are_blocking`, `spec_c06_invalid_status_transition_fails_closed` |
| AC-4 | ✅ | `cargo test -p kamn-core --test data_layer_m11_hardening_readiness` imports M11 symbols from `kamn_core` root exports |
| AC-5 | ✅ | `bash scripts/ci/check_shell_rust_ratio_guardrail.sh ...` and `bash scripts/ci/check_shell_loc_hard_ceiling.sh ...` with Rust-only diff |

## Success Metrics
- M11 hardening readiness contracts are exported through `kamn_core`.
- All ACs map to passing `spec_c0x_*` tests with deterministic reason markers.
- Shell-to-Rust ratio direction remains improved/neutral through Rust-only changes.
