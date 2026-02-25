# Tasks: Issue #5961 - Task: close escaped http_transport mutants from #5932 mutation gate

- Issue: #5961
- Spec: `specs/5961/spec.md`
- Plan: `specs/5961/plan.md`
- Status: Draft
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): add failing tests mapped to C-01..C-04 for escaped mutant surfaces.
- T2 (GREEN / Implementation): implement minimal test helper logic and assertions so all new tests pass.
- T3 (Regression): run targeted kamn-core tests for `kolme_runtime_commit/http_transport`.
- T4 (Mutation): run scoped mutation command and verify C-05 (0 escapes for listed mutants).
- T5 (Process): update issue comments with verification evidence and link PR updates.
