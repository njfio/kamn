# Issue #5509 Spec - R50 Governance-Feature Activity Rebalancing Contracts

- Status: Implemented
- Issue: #5509
- Parent: #5469
- Milestone: R50.20 Governance-feature activity rebalancing contracts

## Problem Statement
R50 records a governance-feature activity imbalance (28 governance commits vs 3 feature commits, 0.11:1), but lacks deterministic remediation targets for future release cycles.

## Scope
In scope:
- Add deterministic governance-feature rebalancing plan markers to `docs/review/gaps-and-issues-r50.md`.
- Add docs-contract tests validating marker presence and arithmetic consistency.
- Update R50 priority/status wording to reflect active rebalancing contracts.

Out of scope:
- Rewriting historical commit history.
- Runtime/API capability changes.

## Acceptance Criteria
- AC-1: R50 review doc defines deterministic governance-feature rebalancing plan markers (baseline, target ratios, target minimum feature commits, derived deltas/caps, target release, status).
- AC-2: Marker arithmetic is internally consistent (baseline totals, derived delta/cap, and ratio consistency).
- AC-3: R50 priority/status wording reflects active rebalancing contract state.
- AC-4: A dedicated docs-contract suite enforces marker presence and consistency.
- AC-5: Targeted tests pass for new and related activity-ratio docs-contract suites.

## Conformance Cases
- C-01 (AC-1): Rebalancing schema/version and required markers exist in R50 report.
- C-02 (AC-2): Derived arithmetic checks pass for commit totals, delta, and cap formulas.
- C-03 (AC-3): R50 priority row reflects active rebalancing contract language.
- C-04 (AC-4): New docs-contract test asserts marker presence and arithmetic consistency.
- C-05 (AC-5): `cargo test -p kamn-core --test review_r50_governance_feature_rebalancing_docs_contract` passes.

## Success Metrics / Observable Signals
- R50 report exposes deterministic governance-feature rebalancing policy markers.
- CI-enforced docs-contract tests fail on marker drift and pass on valid policy state.
