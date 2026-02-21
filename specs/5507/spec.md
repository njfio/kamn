# Issue #5507 Spec - R50 Doc-Contract Suite Consolidation Guardrail Contracts

- Status: Implemented
- Issue: #5507
- Parent: #5469
- Milestone: R50.19 Doc-contract suite consolidation guardrail contracts

## Problem Statement
R50 holds the doc-contract test surface at 82 files, but the surface is still elevated versus the post-consolidation baseline and lacks deterministic anti-regression guardrails.

## Scope
In scope:
- Add deterministic doc-contract consolidation markers to `docs/review/gaps-and-issues-r50.md`.
- Add docs-contract tests that validate marker presence and arithmetic consistency.
- Update R50 status wording to show active consolidation contract execution.

Out of scope:
- Immediate refactor/reduction of the full docs-contract test suite.
- Runtime/API behavior changes.

## Acceptance Criteria
- AC-1: R50 review doc defines deterministic doc-contract consolidation policy markers (baseline, cap, reduction, tranche plan, issue cap, target release, status).
- AC-2: Marker arithmetic is internally consistent (`required_reduction = baseline - cap`; tranche plan covers required reduction).
- AC-3: R50 report narrative and priority summary reflect active consolidation contract status.
- AC-4: A dedicated docs-contract test enforces marker presence and arithmetic consistency.
- AC-5: Targeted tests pass for new and related R50 docs-contract suites.

## Conformance Cases
- C-01 (AC-1): Consolidation policy schema/version and required markers exist in R50 report.
- C-02 (AC-2): Baseline/cap/reduction and tranche arithmetic checks pass.
- C-03 (AC-3): R50 summary text includes active consolidation contract status.
- C-04 (AC-4): New docs-contract test asserts marker presence and arithmetic.
- C-05 (AC-5): `cargo test -p kamn-core --test review_r50_doc_contract_consolidation_docs_contract` passes.

## Success Metrics / Observable Signals
- R50 review artifact carries deterministic consolidation guardrail markers.
- CI-enforced docs-contract test fails on marker drift and passes on valid policy state.
