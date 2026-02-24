# Tasks: Issue #5866 - Service API Durable Persistence Continuity

- Issue: #5866
- Spec: `specs/5866/spec.md`
- Plan: `specs/5866/plan.md`
- Status: Draft
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED): add failing persistence/restart/no-op tests for uncovered mutation families.
- T2 (Implementation): fix durable save/load behavior where gaps are found.
- T3 (Regression): rerun persistence continuity lanes.
- T4 (Verify): clippy/fmt + scoped tests.

## Tier Mapping
| Task | Tier(s) |
|---|---|
| T1 | Integration, Regression, Conformance |
| T2 | Unit, Functional |
| T3 | Regression |
| T4 | Verify |
