# Spec: Issue #6301 - Python LOC tracking in CI shell-surface gates

## Objective

Add deterministic Python LOC accounting to the existing shell-surface CI telemetry path so Python
surface growth/regression is visible in gate outputs alongside script/shell/rust metrics.

## Inputs/Outputs

- Inputs:
  - repository script tree (`scripts/**`) for Python LOC counting.
  - combined shell-surface baseline fixture (`fixtures/ci/combined_shell_surface_trend_baseline.json`).
  - generated combined shell-surface trend report consumed by telemetry collector.
- Outputs:
  - combined trend report includes:
    - `current.python_line_total`
    - `baseline.python_line_total`
    - `deltas.python_line_total`
  - collector output/json includes:
    - `python_line_total`
    - `delta_python_line_total`
  - deterministic validation failures when Python LOC metrics are missing/invalid.

## Boundaries/Non-goals

- In scope:
  - `generate_combined_shell_surface_trend_report.py` metric generation.
  - `collect_shell_rust_loc_telemetry.sh` metric extraction/validation.
  - related contract tests and fixture updates.
  - CI documentation metric list updates.
- Out of scope:
  - changing shell-rust guardrail threshold semantics.
  - adding new policy thresholds for Python LOC in this issue.
  - Python-to-Rust migration work.

## Failure Modes

- FM-1: Python LOC is omitted from report/telemetry output.
- FM-2: Python LOC type drift (non-integer) is accepted by collector.
- FM-3: Delta math is inconsistent with current-baseline values.
- FM-4: Docs/contract descriptions drift from implemented telemetry surface.

## Acceptance Criteria

- AC-1: Combined trend report includes deterministic `python_line_total` for `current`, `baseline`,
  and `deltas`.
- AC-2: Collector output and JSON telemetry include `python_line_total` and
  `delta_python_line_total`.
- AC-3: Collector fails closed on missing/invalid Python LOC metrics with existing metric-type
  validation reason code surface.
- AC-4: Contract tests fail on Python metric omission/type drift and pass when metrics are valid.
- AC-5: CI documentation references Python LOC as part of combined shell-surface telemetry.

## Files To Touch

- `scripts/ci/generate_combined_shell_surface_trend_report.py`
- `scripts/ci/collect_shell_rust_loc_telemetry.sh`
- `scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
- `scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
- `fixtures/ci/combined_shell_surface_trend_baseline.json`
- `docs/ci/strategy.md`
- `docs/ci/ci-cost-and-lane-framework.md`

## Error Semantics

- Maintain existing fail-closed behavior and reason taxonomy versions.
- Reuse existing metric-type error code:
  - `shell_rust_loc_telemetry_metric_type_invalid`
  when Python LOC metrics are missing/invalid.
- No silent defaulting in telemetry output beyond explicit baseline fallback value handling.

## Test Plan

- RED:
  - extend collector/generator contract tests to require Python LOC markers and typed JSON fields.
  - add tampered-report assertions proving collector fails on invalid Python metric types.
- GREEN:
  - implement Python LOC counting in generator and propagate baseline/delta values.
  - propagate Python metrics through collector extraction and marker emission.
- REFACTOR:
  - keep counting logic concise and deterministic.
  - preserve reason taxonomy contract and avoid new error-code churn.
- Verification:
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
