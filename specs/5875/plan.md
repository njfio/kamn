# Plan: Issue #5875 - Immutable Review Docs + Shell LOC Reduction

- Issue: #5875
- Spec: `specs/5875/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. Add RED docs-contract assertions for new immutability policy markers and enforcement semantics.
2. Implement immutability enforcement in `review_r53_docs_contract` using policy-driven release floor and deterministic checks.
3. Consolidate high-duplication shell logic into shared helpers to produce a measurable net shell LOC reduction.
4. Add/adjust shell-surface regression checks to ratchet against this issue's baseline.
5. Run targeted then broader verification suites and capture measured shell/rust deltas.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `docs/review/review-document-freeze.policy`
- `scripts/**` (selected high-duplication shell assets)
- `scripts/ci/check_shell_loc_hard_ceiling.sh` and/or related shell-surface checks (if needed for ratchet wiring)

## Risks + Mitigations
- Risk: Git-history-dependent checks may behave differently in shallow clones.
  - Mitigation: keep primary policy checks deterministic in repository context and scope effective release to new/future docs.
- Risk: Shell consolidation could regress lane behavior.
  - Mitigation: preserve script interfaces and validate via targeted script test suites.
- Risk: LOC reduction may be too small to be meaningful.
  - Mitigation: set explicit target delta in spec and verify with measured report output.

## Interfaces / Contracts
- Review freeze policy markers gain explicit immutability markers:
  - `review_document_immutability_schema_version`
  - `review_document_immutability_effective_release_min`
  - `review_document_immutability_enforcement_mode`
  - `review_document_immutability_max_commits_per_doc`
- Docs contract test enforces marker presence and rule semantics.

## ADR Requirement
- Not required (no dependency/protocol/architecture strategy change).
