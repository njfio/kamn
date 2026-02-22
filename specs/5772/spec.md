# Spec: #5772 Complete R53 Review Marker Contract Coverage

- Issue: #5772
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
`docs/review/gaps-and-issues-r53.md` is incomplete against the marker contract defined in
`docs/review/README.md`. A contract audit shows 70 missing required marker keys for R53 across
snapshot semantics, non-regression ratchets, and post-publication reconciliation schemas.

## Scope
### In Scope
- Add missing required marker blocks/keys to `docs/review/gaps-and-issues-r53.md`.
- Keep marker values consistent with existing R53 snapshot narrative and contract invariants.
- Extend docs-contract tests to enforce R53 marker presence and consistency fail-closed.
- Add lifecycle artifacts and milestone tracking updates.
- Perform compensating archived issue-spec pair cleanup to preserve top-level `specs/` cap (`<= 693`) after adding `specs/5772`.

### Out of Scope
- Rewriting R53 narrative assessment beyond marker-completion sections.
- CI/workflow changes.

## Acceptance Criteria
### AC-1 R53 required marker keys present
Given README marker contracts for R43+/R50+/R52+,
When R53 review artifact is completed,
Then all required R53 marker keys are present in `gaps-and-issues-r53.md`.

### AC-2 Marker values are internally consistent
Given marker invariants (count/ratio, pre/post deltas, alignment across sections),
When docs-contract tests parse R53 markers,
Then invariants hold and status fields are deterministic.

### AC-3 Historical snapshot rows preserved
Given existing R53 narrative rows,
When completion markers are added,
Then baseline summary/priority table wording remains unchanged.

### AC-4 Fail-closed contract enforcement
Given marker drift/removal,
When docs-contract tests run,
Then R53 contract tests fail deterministically.

### AC-5 Specs cap preserved
Given one new lifecycle spec directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): R53 contains all required key groups (snapshot semantics, non-regression ratchets, post-publication reconciliations).
- C-02 (AC-2): R53 marker ratio/delta/status invariants pass test assertions.
- C-03 (AC-3): existing R53 summary/priority row texts remain unchanged.
- C-04 (AC-4): `cargo test -p kamn-core --test review_r53_docs_contract` (new).
- C-05 (AC-5): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-06 (AC-5): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-07 (AC-1..AC-5): `cargo fmt --all --check`.
- C-08 (AC-1..AC-5): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- New R53 docs-contract test lane passes and fails on key drift.
- Missing required marker count reduced from 70 to 0.
- `specs/` top-level directory count remains at or below cap.
