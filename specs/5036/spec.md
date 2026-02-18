# Issue #5036 Spec

- Title: Subtask: M7 telemetry aggregate correctness and billing reconciliation regressions
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5023` requires deterministic correctness for telemetry rollups
and billing usage projection. The existing M7 surface computes aggregates but
does not expose an explicit billing reconciliation contract to verify an
external daily statement against projected owner usage, leaving regression
detection implicit.

## Acceptance Criteria
- AC-1: M7 exposes deterministic owner billing daily reconciliation API that
  compares statement totals vs projected totals for a daily bucket.
- AC-2: Reconciliation reports deterministic `Match`/`Mismatch` decisions with
  stable reason markers and explicit projected/statement totals.
- AC-3: Reconciliation fails closed on owner-scope violations and invalid
  bucket alignment.
- AC-4: Existing hourly/daily/network aggregate and billing projection behavior
  remains deterministic and passing.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add M7 billing reconciliation contracts (input/report/decision/reason codes)
  in `data_layer_m7_timeseries_telemetry`.
- Add conformance tests for reconciliation match, mismatch, owner-scope denial,
  and invalid bucket alignment.
- Validate scoped/full regression and shell guardrail evidence.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI workflow or shell-script changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Owner daily statement equals projected totals | Reconciliation decision is `Match` with stable match reason marker |
| C-02 | AC-2 | Conformance | Owner daily statement differs from projected totals | Reconciliation decision is `Mismatch` with stable mismatch reason marker and echoed totals |
| C-03 | AC-3 | Regression | Cross-owner reconciliation query | Owner-scope violation error with stable reason marker |
| C-04 | AC-3 | Regression | Non-daily bucket start alignment | Fail-closed typed invalid bucket error |
| C-05 | AC-4 | Conformance | Existing M7 aggregate/projection suites | Existing deterministic aggregate tests remain green |
| C-06 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5036.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5036.json`

## Success Metrics
- Reconciliation API deterministically reports match/mismatch outcomes with
  stable markers and explicit totals.
- All M7 conformance cases pass in `data_layer_m7_timeseries_telemetry` suite.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
