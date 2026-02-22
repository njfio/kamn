# Plan: Issue #5793 — Resolve All Unresolved Items in R54 Review

- Issue: #5793
- Spec: `specs/5793/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Add RED checks for R54 unresolved-item closure markers and tracked-only spec-dir semantics.
2. Update `docs/review/gaps-and-issues-r54.md` to carry full closure marker family and moratorium-compliant headings.
3. Extend `review_r53_docs_contract.rs` with fail-closed R54 closure invariant checks.
4. Update `review_r50_spec_volume_remediation_docs_contract.rs` to compute current spec-dir count from tracked files (`git ls-files specs`) and add regression coverage for untracked contamination.
5. Update `docs/review/README.md` marker contract docs for new R54 unresolved-item closure schema.
6. Preserve spec cap by deleting one archived pointer-only `specs/<id>/ARCHIVED.md` directory in the same change set.
7. Update milestone index with the completed slice.

## Affected Modules
- `docs/review/gaps-and-issues-r54.md`
- `docs/review/README.md`
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `specs/5793/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: R54 doc content may violate existing R54+ moratorium checks once tracked.
  - Mitigation: remove disallowed heading patterns and validate via `review_r53_docs_contract`.
- Risk: spec cap breach after adding `specs/5793`.
  - Mitigation: remove one archived pointer-only spec directory in same PR.
- Risk: tracked-only counting shell command parse drift.
  - Mitigation: parse `git ls-files specs` deterministically and test with untracked temp dir.

## Interfaces / Contracts
- New R54 unresolved-item closure marker schema in review docs.
- Existing governance-remediation budget policy contract reused for R54 governance closure.
- Updated spec-volume non-regression runtime count semantics: tracked top-level spec dirs only.

## ADR
- None required.
