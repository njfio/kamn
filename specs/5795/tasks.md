# Tasks: Issue #5795 — Execute Merged-Only Remote Branch Cleanup to <=50 Heads

- Issue: #5795
- Spec: `specs/5795/spec.md`
- Plan: `specs/5795/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (Baseline): capture current remote head count and target delta.
2. T2 (Safety): compute protected/open-PR exclusion set and merged-lineage candidates (ancestor or merged PR head).
3. T3 (Implement): delete sufficient merged-lineage remote branches to reach <=50.
4. T4 (Verify): recount heads and validate merged-only deletion invariant.
5. T5 (Lifecycle): update milestone index + finalize spec/task statuses.

## AC/Tier Mapping
- AC-1: T1, T3, T4 (Functional)
- AC-2: T2, T4 (Regression)
- AC-3: T4, T5 (Integration)
- AC-4: T5 (Functional)
