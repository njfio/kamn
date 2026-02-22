# Issue #5551 Spec - R50 Spec-Volume Non-Regression Ratchet Guardrail Enforcement

- Status: Reviewed
- Issue: #5551
- Parent: None
- Milestone: R50.41 Spec-volume non-regression ratchet guardrail enforcement

## Problem Statement
R50 documents a breached spec-volume guardrail and a multi-tranche remediation plan, but no active non-regression gate prevents additional spec-volume growth while remediation is in progress.

## Scope
In scope:
- Add deterministic R50 non-regression ratchet markers for current baseline/max spec-dir and ratio values.
- Extend existing spec-volume remediation docs-contract lane to validate current repository counts against ratchet limits.
- Document ratchet schema and invariants in `docs/review/README.md`.

Out of scope:
- Deleting/moving historical spec directories.
- Runtime/service/protocol behavior changes.

## Acceptance Criteria
- AC-1: `docs/review/gaps-and-issues-r50.md` includes deterministic non-regression ratchet markers for spec-dir and ratio max values.
- AC-2: `review_r50_spec_volume_remediation_docs_contract.rs` enforces non-regression by computing current spec-dir and module counts and checking against ratchet markers.
- AC-3: `docs/review/README.md` documents ratchet marker schema and invariants.
- AC-4: Existing review docs-contract lanes remain green with the extended ratchet checks.
- AC-5: Issue lifecycle artifacts (`spec.md`, `plan.md`, `tasks.md`) are complete and spec status advances to Implemented.

## Conformance Cases
- C-01 (AC-1): R50 artifact contains ratchet schema/version marker and max markers.
- C-02 (AC-2): integration test computes current spec-dir/module counts and verifies current ratio and counts do not exceed ratchet maxima.
- C-03 (AC-2): ratchet baseline/max consistency is validated (`baseline <= max`).
- C-04 (AC-3): README includes ratchet marker schema and invariant statements.
- C-05 (AC-4): existing review marker lanes pass without regressions.

## Success Metrics / Observable Signals
- Future PRs cannot increase spec-volume beyond ratchet maxima without explicit policy change.
- Spec-volume remediation remains bounded while preserving existing review marker contracts.
