# Issue #5555 Spec - R50 Governance-Feature Activity Non-Regression Ratchet Enforcement

- Status: Implemented
- Issue: #5555
- Parent: None
- Milestone: R50.43 Governance-feature activity non-regression ratchet enforcement

## Problem Statement
R50 includes governance-feature rebalancing plan markers, but lacks an explicit non-regression ratchet that locks baseline minimum feature ratio and maximum governance ratio for marker-contract drift detection.

## Scope
In scope:
- Add deterministic R50 governance-feature non-regression ratchet markers.
- Extend the existing governance-feature docs-contract lane with ratchet-bound assertions.
- Document ratchet schema and invariants in `docs/review/README.md`.

Out of scope:
- Runtime/protocol behavior changes.
- CI workflow changes.

## Acceptance Criteria
- AC-1: `docs/review/gaps-and-issues-r50.md` includes governance-feature non-regression ratchet markers (schema, min feature ratio, max governance ratio).
- AC-2: `review_r50_governance_feature_rebalancing_docs_contract.rs` enforces current ratio markers against ratchet bounds.
- AC-3: `docs/review/README.md` documents governance-feature non-regression ratchet schema and invariants.
- AC-4: Existing review docs-contract lanes remain green.
- AC-5: Lifecycle artifacts are complete and spec status advances to Implemented.

## Conformance Cases
- C-01 (AC-1): R50 artifact contains required ratchet markers.
- C-02 (AC-2): integration lane validates current `governance_activity_commit_ratio` <= ratchet max.
- C-03 (AC-2): integration lane validates current `feature_activity_commit_ratio` >= ratchet min.
- C-04 (AC-3): README includes ratchet schema and invariant strings.
- C-05 (AC-4): release review activity-ratio and R50 review docs-contract lanes pass.

## Success Metrics / Observable Signals
- Governance-feature ratio drift below baseline feature share / above baseline governance share is fail-closed by docs-contract tests.
- Existing review marker contracts remain coherent.
