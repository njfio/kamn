# Tasks: Issue #5781 — Extend Opt-In Live S-04 Task Lifecycle Execution Across E2E Drivers

- Issue: #5781
- Spec: `specs/5781/spec.md`
- Plan: `specs/5781/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (Functional): add RED tests for S-04 live dispatch/fail-closed behavior in all three driver modules.
2. T2 (Implementation): implement scenario-aware live probe routing for `S-01` + `S-04` in each driver.
3. T3 (Regression): ensure existing S-01 and non-live tests remain green.
4. T4 (Quality): run targeted harness tests, `cargo fmt --all --check`, and targeted clippy.
5. T5 (Integration): run workspace gate command and update lifecycle/milestone artifacts.

## AC/Tier Mapping
- AC-1: T1, T2, T3 (Functional/Regression)
- AC-2: T1, T2, T3 (Functional/Regression)
- AC-3: T1, T2, T3 (Functional/Regression)
- AC-4: T3, T4, T5 (Regression/Integration)
