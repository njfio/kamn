# Issue #5497 Plan - R49 Marker/Provenance Reconciliation

## Approach
1. Update R49 post-publication branch-count values from `51` to `50`.
2. Replace reconciliation marker issue id from `5491` to `5495`.
3. Update docs-contract assertions for branch count and reconciliation marker.
4. Verify with targeted docs-contract test and format checks.

## Affected Modules
- `docs/review/gaps-and-issues-r49.md`
- `crates/kamn-core/tests/review_r49_docs_contract.rs`
- `specs/milestones/r50-14-r49-branch-count-marker-reconciliation-after-stale-trim/index.md`
- `specs/5497/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: accidental drift between prose and markers.
  - Mitigation: update both and enforce with docs-contract assertions.

## Interfaces / Contracts
- Documentation contract only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r49_docs_contract`
- `cargo fmt --check`
