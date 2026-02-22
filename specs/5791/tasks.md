# Tasks: Issue #5791 — Enforce R54+ Governance-Remediation Commit-Budget Docs-Contract

- Issue: #5791
- Spec: `specs/5791/spec.md`
- Plan: `specs/5791/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (RED/Functional): add governance-remediation budget test and capture failing run.
2. T2 (GREEN/Implementation): add policy baseline file and pass targeted test.
3. T3 (Regression/Integration): run review contract lanes (`review_r53_docs_contract`, `review_r50_spec_volume_remediation_docs_contract`).
4. T4 (Quality): run fmt/clippy scoped lanes.
5. T5 (Lifecycle): update active milestone index and preserve spec-dir cap.

## AC/Tier Mapping
- AC-1: T1, T2 (Functional)
- AC-2: T1, T2, T3 (Regression)
- AC-3: T3, T4 (Integration)
