# Issue #5553 Spec - R50 Doc-Contract Test-File Non-Regression Ratchet Enforcement

- Status: Reviewed
- Issue: #5553
- Parent: None
- Milestone: R50.42 Doc-contract test-file non-regression ratchet enforcement

## Problem Statement
R50 doc-contract consolidation markers encode target reductions, but no active contract currently enforces a non-regression ceiling on doc-contract test-file count during remediation.

## Scope
In scope:
- Add deterministic R50 non-regression ratchet markers for doc-contract test-file count.
- Extend the existing R50 doc-contract consolidation docs-contract lane with dynamic count enforcement.
- Document the ratchet schema/invariants and counting formula in `docs/review/README.md`.

Out of scope:
- Deleting or moving test files.
- Runtime/service/protocol changes.

## Acceptance Criteria
- AC-1: `docs/review/gaps-and-issues-r50.md` includes deterministic doc-contract non-regression ratchet markers (schema version, baseline count, max count, counting formula key).
- AC-2: `review_r50_doc_contract_consolidation_docs_contract.rs` computes current doc-contract test-file count from the declared formula and enforces `current <= max`.
- AC-3: `docs/review/README.md` documents R50+ doc-contract non-regression ratchet marker schema and invariants.
- AC-4: Existing review docs-contract lanes remain green after migration.
- AC-5: Spec lifecycle artifacts are complete and status advances to Implemented.

## Conformance Cases
- C-01 (AC-1): R50 artifact exposes ratchet markers and a deterministic counting formula marker.
- C-02 (AC-2): integration test computes current count using the declared formula and validates non-regression ceiling.
- C-03 (AC-2): baseline/max marker consistency is validated (`baseline <= max`).
- C-04 (AC-3): README includes ratchet schema and invariant text.
- C-05 (AC-4): R50 governance-loop, spec-volume, and activity-ratio review lanes remain green.

## Success Metrics / Observable Signals
- Doc-contract test-file count cannot grow above the ratchet maximum without an explicit contract update.
- Consolidation remediation remains bounded and observable in existing review marker lanes.
