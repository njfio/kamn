# Plan: #5723 Execute R52 Spec-Volume Remediation Tranche-4 (14-Dir Reduction)

## Approach
1. RED: update existing docs-contract expectations to tranche-4 values and capture failing run.
2. Implementation: remove 14 archived issue-spec pairs (pointer + payload).
3. Update `specs/archive/index.md` rows and `archived_issue_count`.
4. Refresh R50 + R52 review markers to tranche-4 values.
5. GREEN + verification: targeted docs-contract suite, archive policy checker, companion regression suites, fmt/clippy.

## Affected Artifacts
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `docs/review/gaps-and-issues-r50.md`
- `docs/review/gaps-and-issues-r52.md`
- `specs/archive/index.md`
- selected archived pointer/payload directories under `specs/` and `specs/archive/`

## Risks and Mitigations
- Risk: marker values drift under concurrent mainline changes.
  - Mitigation: branch from `origin/main`, reconcile counts with deterministic pre/post evidence.
- Risk: archive index count mismatch.
  - Mitigation: remove rows and update header count atomically; run archive-policy checker.
- Risk: docs-contract expectations stale.
  - Mitigation: RED first, then align code/docs and re-run targeted suite.

## Interfaces / Contracts
- Marker schema: `kamn.review.spec-volume-post-publication-reduction.v1`
- Required invariant: `pre_count - deleted_count = post_count`
- Ratchet invariant: current top-level `specs/` dir count <= non-regression cap

## ADR
No ADR required (no dependency, architecture, or protocol changes).
