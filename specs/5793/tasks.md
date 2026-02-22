# Tasks: Issue #5793 — Resolve All Unresolved Items in R54 Review

- Issue: #5793
- Spec: `specs/5793/spec.md`
- Plan: `specs/5793/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (RED): add/extend tests for R54 unresolved-item closure markers and tracked-only spec-dir semantics.
2. T2 (GREEN): update `docs/review/gaps-and-issues-r54.md` with closure markers and moratorium-compliant section wording.
3. T3 (GREEN): implement tracked-only spec-dir counting and untracked-contamination regression guard in spec-volume docs-contract tests.
4. T4 (Regression): run `review_r53_docs_contract` and `review_r50_spec_volume_remediation_docs_contract`.
5. T5 (Quality): run fmt/clippy scoped lanes.
6. T6 (Lifecycle): update milestone index, preserve spec cap, finalize spec/task statuses.

## AC/Tier Mapping
- AC-1: T1, T2 (Functional)
- AC-2: T1, T2, T4 (Regression)
- AC-3: T1, T2, T4 (Regression)
- AC-4: T1, T2 (Functional)
- AC-5: T1, T3, T4 (Regression)
- AC-6: T4, T5 (Integration)
