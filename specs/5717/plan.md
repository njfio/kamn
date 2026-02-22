# Plan: #5717 Execute R52 Spec-Volume Remediation Tranche-2 (14-Dir Reduction)

## Approach
1. Add RED coverage for tranche-2 markers in the existing spec-volume docs-contract test suite.
2. Capture pre-count evidence and remove 14 archived issue-spec pairs (pointer + payload).
3. Update `specs/archive/index.md` rows and `archived_issue_count`.
4. Update review artifacts (`gaps-and-issues-r52.md`, `gaps-and-issues-r50.md`) with tranche-2 and
   non-regression ratchet values.
5. Run targeted test gates, archive-policy checker, and quality gates (`fmt`/`clippy`).

## Affected Modules/Artifacts
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `docs/review/gaps-and-issues-r52.md`
- `docs/review/gaps-and-issues-r50.md`
- `specs/archive/index.md`
- archive/pointer directories under `specs/` and `specs/archive/`

## Risks and Mitigations
- Risk: concurrent mainline changes can shift pre/post evidence values.
  - Mitigation: branch from `origin/main`; treat tranche markers as point-in-time execution values.
- Risk: archive index drift (count or rows mismatch).
  - Mitigation: update rows + `archived_issue_count`, run archive-policy checker.
- Risk: docs-contract tests become stale/hardcoded incorrectly.
  - Mitigation: run RED first, then update all markers/tests together and re-run targeted suites.

## Interfaces / Contracts
- Marker contract schema:
  - `kamn.review.spec-volume-post-publication-reduction.v1`
- Deterministic invariant:
  - `pre_count - deleted_count = post_count`
- Non-regression ratchet invariant:
  - current `specs/` directory count <= `r50_review_spec_volume_non_regression_spec_dir_max`

## ADR
No ADR required (no dependency, architecture, or protocol changes).
