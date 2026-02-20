# Issue #5315 Spec

- Title: Mitigation: offset shell-surface delta from #4000 performance fixture matrix
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement
Issue #4000 correctly added deterministic workload fixture support, but it increased shell LOC. This mitigation must reclaim at least that shell-surface increase without regressing the fixture-matrix behavior introduced in #4000.

## Acceptance Criteria
- AC-1: Consolidate shell-surface governance logic so mitigation shell LOC delta is at least `-133` LOC.
- AC-2: Net shell LOC delta across #4000 + #5315 is non-positive.
- AC-3: Net shell-to-rust ratio delta across #4000 + #5315 is non-positive.
- AC-4: #4000 fixture-matrix behavior remains green (`generate_performance_smoke_report.sh` + associated tests).
- AC-5: Shell-rust ratio guardrail behavior remains contract-equivalent after shell-surface consolidation.

## Scope
In scope:
- Add shell conformance checks for minimal shell surface on `check_shell_rust_ratio_guardrail.sh`.
- Move ratio-guardrail logic to a Python implementation while preserving the shell entrypoint.
- Run targeted CI/contract checks for ratio-guardrail behavior and #4000 fixture coverage.

Out of scope:
- Reversing #4000 behavior.
- New workflow topology or dependency changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | `scripts/ci/check_shell_rust_ratio_guardrail.sh` | shell entrypoint is thin delegator (`<=20` lines) to python checker |
| C-02 | AC-1 | Functional | LOC diff for changed shell files | shell LOC mitigation <= `-133` |
| C-03 | AC-2/AC-3 | Regression | pre/post shell-ratio metrics + known #4000 delta | combined shell LOC and ratio deltas are non-positive |
| C-04 | AC-4 | Regression | `test_generate_performance_smoke_report.sh` | #4000 performance fixture tests pass unchanged |
| C-05 | AC-5 | Integration | `test_check_shell_rust_ratio_guardrail.sh` | ratio guardrail checker parity tests remain green |

## Test Mapping
- `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- `bash scripts/ci/test_generate_performance_smoke_report.sh`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json <tmp>`

## Success Metrics
- Mitigation shell LOC reclaim is at least `133` lines.
- No regression in performance fixture-matrix behavior from #4000.
- No regression in non-kolme wrapper dispatch contracts.
