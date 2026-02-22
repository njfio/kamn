# Plan: Issue #5787 — Freeze R53 Review Artifact with Fail-Closed Docs-Contract Guard

- Issue: #5787
- Spec: `specs/5787/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Add RED freeze-contract test in `review_r53_docs_contract.rs` that requires a freeze metadata file and validates baseline invariants.
2. Run targeted freeze test expecting failure due missing metadata file.
3. Create `docs/review/gaps-and-issues-r53.freeze` with deterministic baseline markers.
4. Re-run targeted and full R53 docs-contract tests to GREEN.
5. Run cap-sensitive R50 spec-volume remediation docs-contract lane.
6. Update active milestone (`r52`) index with completed slice marker and preserve spec-dir cap.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `docs/review/gaps-and-issues-r53.freeze`
- `specs/5787/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: adding `specs/5787/` can exceed the spec-dir non-regression cap.
  - Mitigation: remove one obsolete archived pointer-only spec directory in same change set.
- Risk: freeze fingerprint algorithm instability.
  - Mitigation: use deterministic in-test FNV-1a 64 implementation with explicit baseline hex value.

## Interfaces / Contracts
- New docs freeze contract file: `docs/review/gaps-and-issues-r53.freeze`
- Existing R53 review contract test module remains source of fail-closed enforcement.

## ADR
- None required (no dependency/protocol/architecture decision changes).
