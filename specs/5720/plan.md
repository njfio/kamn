# Plan: #5720 Execute R52 Spec-Volume Remediation Tranche-3 (14-Dir Reduction)

## Approach
1. Perform RED by updating existing spec-volume docs-contract tests to tranche-3 expected markers.
2. Capture pre-count and remove 14 additional archived issue-spec pairs (pointer + payload).
3. Update `specs/archive/index.md` rows and `archived_issue_count`.
4. Refresh R50 + R52 review markers to tranche-3 values.
5. Run targeted docs-contract tests, archive policy checker, regression suites, fmt, and clippy.

## Affected Artifacts
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `docs/review/gaps-and-issues-r50.md`
- `docs/review/gaps-and-issues-r52.md`
- `specs/archive/index.md`
- selected `specs/<id>/` and `specs/archive/<id>/` directories

## Risks and Mitigations
- Risk: marker values drift with concurrent mainline changes.
  - Mitigation: execute on `origin/main` baseline and rebase/reconcile if needed.
- Risk: archive index drift from row/count mismatch.
  - Mitigation: update rows + count atomically and run archive policy checker.
- Risk: docs-contract expectation mismatch.
  - Mitigation: RED first, then align docs/tests and verify with targeted suite.

## Interfaces / Contracts
- Schema: `kamn.review.spec-volume-post-publication-reduction.v1`
- Invariant: `pre_count - deleted_count = post_count`
- Non-regression invariant: current spec-dir count <= non-regression spec_dir_max

## ADR
No ADR required (no dependency/architecture/protocol changes).
