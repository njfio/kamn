# Tasks: Issue #5958 - Task: complete full mutation gate for #5932 networking hardening

- Issue: #5958
- Spec: `specs/5958/spec.md`
- Plan: `specs/5958/plan.md`
- Status: Draft
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): verify PR #5957 lacks full mutation evidence for `kamn-core`/`kamn-node`.
- T2 (GREEN / Mutation): run full mutation scope for `kamn-core` and `kamn-node` without sharding.
- T3 (Triage): extract totals and disposition escaped mutants.
- T4 (Regression): add catching tests/fixes for escaped mutants when feasible.
- T5 (Verify): rerun relevant mutation/tests and confirm totals.
- T6 (Process): post mutation evidence on PR #5957 and update #5958 logs/status.
