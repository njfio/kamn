# Issue #5503 Spec - R50 Governance-Loop Mitigation Contracts

- Status: Accepted
- Issue: #5503
- Parent: #5469
- Milestone: R50.17 Governance-loop mitigation contracts for review artifacts

## Problem Statement
R50 identifies a self-referential governance loop where review-marker reconciliation produced governance-only churn (7 issues, 20 commits, 10 spec dirs) without capability value.

## Scope
In scope:
- Add deterministic mitigation policy markers to `docs/review/gaps-and-issues-r50.md`.
- Add docs-contract tests asserting mitigation markers and arithmetic consistency.
- Encode a bounded reconciliation policy (single follow-up issue cap and constrained spec-artifact growth).

Out of scope:
- Deleting historical spec directories.
- Runtime/API behavior changes.

## Acceptance Criteria
- AC-1: R50 review doc includes deterministic point-in-time marker semantics for branch-count and review metrics.
- AC-2: R50 review doc includes bounded reconciliation policy markers (issue cap and spec-artifact cap).
- AC-3: R50 review doc includes explicit spec-volume remediation arithmetic markers derived from baseline and target ratio.
- AC-4: New docs-contract tests validate marker presence and arithmetic consistency.
- AC-5: Targeted tests pass.

## Conformance Cases
- C-01 (AC-1): marker `r50_review_marker_semantics=point_in_time_snapshot` exists.
- C-02 (AC-2): markers `r50_review_reconciliation_followup_issue_cap=1` and `r50_review_reconciliation_spec_artifact_cap=1` exist.
- C-03 (AC-3): markers for baseline spec dirs, modules, target ratio, and required reduction are consistent (`required_reduction = baseline - floor(target_ratio * modules)`).
- C-04 (AC-4): docs-contract test parses numeric markers and verifies derived values.
- C-05 (AC-5): `cargo test -p kamn-core --test review_r50_governance_loop_mitigation_docs_contract` passes.

## Success Metrics / Observable Signals
- R50 artifact includes deterministic mitigation policy markers.
- Governance-loop mitigation math is contract-enforced in CI tests.
