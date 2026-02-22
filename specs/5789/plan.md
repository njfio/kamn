# Plan: Issue #5789 — Enforce R54+ Post-Publication Reconciliation Moratorium Docs-Contract

- Issue: #5789
- Spec: `specs/5789/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Add moratorium policy test to `review_r53_docs_contract.rs` that scans `docs/review/gaps-and-issues-r*.md` for release >=54.
2. RED: validate targeted moratorium lane behavior.
3. Implement filter logic and fail-closed checks for headings and marker keys.
4. Run targeted and integration docs-contract lanes.
5. Update active milestone index and preserve spec-dir cap by removing one archived pointer-only spec directory.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `specs/5789/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: adding `specs/5789/` can exceed spec-dir cap.
  - Mitigation: remove one archived pointer-only spec directory in same change set.
- Risk: over-broad matching catches narrative prose.
  - Mitigation: scope checks to markdown heading lines and marker-key lines only.

## Interfaces / Contracts
- Docs-contract policy applies to future review files with release number >=54.
- Existing R53 freeze contract remains source of truth for R53 artifact immutability.

## ADR
- None required.
