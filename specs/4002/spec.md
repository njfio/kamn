# Issue #4002 Spec

- Title: Subtask: implement low-cost ci smoke performance checker with deterministic pass-fail taxonomy
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement
The repository has a smoke threshold checker command, but issue #4002 requires an explicit deterministic pass/fail reason-taxonomy contract plus selector wiring checks that guarantee fast-gate keeps smoke-only scope and excludes deep/local-heavy entries.

## Acceptance Criteria
- AC-1: CI smoke checker emits deterministic status + reason taxonomy markers for pass and fail paths.
- AC-2: Checker fails closed on threshold/report contract violations and selector/workflow drift.
- AC-3: CI selector contract includes required checker path and excludes deep-lane checker entry in fast-gate.
- AC-4: Unit, Functional, Integration, Regression, and Performance tests are present and passing.
- AC-5: `docs/ci/strategy.md` documents threshold + exclusion markers for the checker contract.

## Scope
In scope:
- Extend performance smoke checker contract outputs with deterministic reason taxonomy markers.
- Enforce fast-mode selector/workflow checker-entry and exclusion invariants.
- Add Rust governance contract tests for checker pass/fail/selector drift/runtime budget.
- Update CI strategy docs with checker threshold/exclusion markers.

Out of scope:
- Changing benchmark fixture workload values.
- Adding deep-lane execution to ci-fast-gate.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 120
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0016
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | valid smoke report + baseline selector/workflow files | checker exits pass with deterministic taxonomy markers and `reason_codes_value=none` |
| C-02 | AC-2 | Functional | smoke report with threshold breach | checker exits fail with deterministic threshold reason code |
| C-03 | AC-3 | Integration | selector/workflow fixture containing forbidden deep entry or missing required entry | checker exits fail with selector/workflow drift reason code |
| C-04 | AC-2 | Regression | malformed report contract marker set | checker exits fail with deterministic report-contract reason code |
| C-05 | AC-4 | Performance | baseline checker invocation | checker runtime remains within fast-gate budget cap |

## Test Mapping
- `cargo test -p kamn-core --test performance_ci_smoke_governance_contract -- --nocapture`
- `bash scripts/ci/test_check_performance_thresholds.sh`

## Success Metrics
- Checker reason taxonomy is deterministic across pass/fail paths.
- Fast-gate selector/workflow drift is detected fail-closed.
- CI strategy docs include enforceable threshold + exclusion markers for performance smoke governance.
