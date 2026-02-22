# Tasks: Issue #5783 — Extend Opt-In Live S-06 Proof-Verification Execution Across E2E Drivers

- Issue: #5783
- Spec: `specs/5783/spec.md`
- Plan: `specs/5783/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (Functional): add RED tests for S-06 live dispatch/fail-closed behavior in all driver modules.
2. T2 (Implementation): implement scenario-aware S-06 live probe routing and dedicated helpers in each driver.
3. T3 (Regression): preserve existing S-01/S-04 live behavior and non-live deterministic behavior.
4. T4 (Quality): run targeted harness tests, `cargo fmt --all --check`, and targeted clippy.
5. T5 (Mutation/Integration): run in-diff mutation gate and workspace gate, then update lifecycle/milestone artifacts.

## AC/Tier Mapping
- AC-1: T1, T2, T3 (Functional/Regression)
- AC-2: T1, T2, T3 (Functional/Regression)
- AC-3: T1, T2, T3 (Functional/Regression)
- AC-4: T3, T4, T5 (Regression/Integration)
