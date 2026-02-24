# Tasks: Issue #5875 - Immutable Review Docs + Shell LOC Reduction

- Issue: #5875
- Spec: `specs/5875/spec.md`
- Plan: `specs/5875/plan.md`
- Status: Draft
- Last Updated: 2026-02-24

## Ordered Tasks
- T1 (RED / Functional): extend `review_r53_docs_contract` to require new immutability policy markers; run targeted test and capture expected failure.
- T2 (GREEN / Implementation): add immutability policy markers and enforcement logic for in-scope review docs.
- T3 (RED / Integration): add shell LOC ratchet assertion against baseline and confirm failing state prior to shell consolidation.
- T4 (GREEN / Implementation): consolidate/refactor selected shell scripts to reduce tracked shell LOC while preserving interfaces.
- T5 (Regression): run targeted script tests and docs-contract tests covering affected paths.
- T6 (Verify): run `cargo fmt --check`, scoped clippy/tests, and shell-surface metric scripts; collect delta evidence for PR.
