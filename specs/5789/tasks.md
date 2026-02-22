# Tasks: Issue #5789 — Enforce R54+ Post-Publication Reconciliation Moratorium Docs-Contract

- Issue: #5789
- Spec: `specs/5789/spec.md`
- Plan: `specs/5789/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (RED/Functional): add moratorium policy test in `review_r53_docs_contract.rs` and run targeted lane.
2. T2 (GREEN/Implementation): implement heading/marker-key rejection logic for R54+ review docs.
3. T3 (Regression/Integration): run `review_r53_docs_contract` and `review_r50_spec_volume_remediation_docs_contract`.
4. T4 (Quality): run formatting and targeted clippy lane.
5. T5 (Lifecycle): update active milestone index and preserve spec-dir cap.

## AC/Tier Mapping
- AC-1: T1, T2 (Functional)
- AC-2: T1, T2, T3 (Regression)
- AC-3: T3, T4 (Integration)
