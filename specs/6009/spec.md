# Spec: Issue #6009 - Refresh shell test surface ratio baseline

- Issue: #6009
- Status: Implemented
- Type: task
- Priority: P2
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
`cargo test -p kamn-core --test shell_test_surface_ratio_policy` fails because the baseline fixture in `fixtures/ci/shell_test_surface_ratio_baseline.env` is stale versus current tracked test-file counts. This blocks PR merge gates even when ratio policy is improving.

## Scope
In scope:
- Recompute current shell/rust test-file counts from repository source.
- Refresh baseline fixture values to match current source-of-truth counts.
- Keep threshold contract strict (`allowed_shell_test_file_delta_max=0`, no waiver).
- Verify shell surface ratio policy test passes with updated baseline.

Out of scope:
- Relaxing thresholds.
- Adding waivers.
- Changing shell/rust counting logic.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: Baseline fixture values exactly match current test-file counts and ratio.
- AC-2: Shell surface ratio gate passes with unchanged strict thresholds and no waiver.
- AC-3: Reason taxonomy/schema markers remain unchanged.

## Conformance Cases
- C-01 (Unit, AC-1): `functional_shell_test_surface_ratio_non_regression_gate` passes after baseline refresh.
- C-02 (Regression, AC-2): threshold fixture still declares `allowed_shell_test_file_delta_max=0` and `waiver_file=None`.
- C-03 (Unit, AC-3): `unit_fixtures_declare_expected_schema_markers` continues to pass.

## Success Metrics / Observable Signals
- CI merge-gate test `-p kamn-core --test shell_test_surface_ratio_policy` is green.
- No new waiver or threshold weakening appears in diff.
