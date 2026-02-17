# Spec — Issue #3967

- Title: Task: enforce shell-surface decline trajectory and wrapper-duplication governance checks
- Parent: Parent story: #3964
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Reviewed
- Priority: P1

## Objective

Refresh the combined shell-surface trend baseline to current repository reality and document a deterministic baseline-refresh workflow so script-scope fast-gate checks remain enforceable and maintainable.

## Problem Statement

The committed combined shell-surface baseline drifted from current script inventory, causing fail-closed policy exits (`combined_shell_surface_shell_line_total_delta_fail_exceeded`) even for unrelated script-scope changes.

## Scope

In scope:
- refresh `fixtures/ci/combined_shell_surface_trend_baseline.json` to current measured metrics
- document deterministic baseline refresh workflow markers in CI strategy docs
- add docs-contract coverage for the refresh workflow markers

Out of scope:
- threshold policy redesign
- wrapper-family migration implementation

## Acceptance Criteria

- AC-1: Combined shell-surface baseline fixture matches current measured script/rust metrics used by fast-gate checker flows.
- AC-2: CI strategy docs include deterministic baseline refresh command and trigger markers.
- AC-3: Regression coverage enforces baseline refresh workflow documentation and policy checker scripts remain green.

## Conformance Cases

- C-01 (AC-1, Functional): `bash scripts/ci/generate_combined_shell_surface_trend_report.sh --output-json /tmp/combined-shell-surface-trend-report.json`
- C-02 (AC-1, Functional): `bash scripts/ci/check_combined_shell_surface_trend_policy.sh --report-file /tmp/combined-shell-surface-trend-report.json --threshold-file fixtures/ci/combined_shell_surface_trend_thresholds.json --output-json /tmp/combined-shell-surface-trend-policy-report.json`
- C-03 (AC-2, Conformance): `cargo test -p kamn-core --test ci_strategy_docs doc_contains_combined_shell_surface_baseline_refresh_workflow_markers -- --exact`
- C-04 (AC-3, Regression): `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`

## Success Metrics / Signals

- Script-scope fast-gate checks fail only on active drift, not stale baseline artifacts.
- Baseline refresh process is explicitly documented and enforced by docs-contract tests.

