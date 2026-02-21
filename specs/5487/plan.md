# Issue #5487 Plan - Reconcile R49 Status-Highlight Branch Count

## Approach
1. Update status-highlight line in `docs/review/gaps-and-issues-r49.md` from `50` to post-publication revalidated `52`.
2. Extend `crates/kamn-core/tests/review_r49_docs_contract.rs` to assert reconciled status-highlight text.
3. Run targeted docs-contract test + fmt + strict clippy.

## Affected Modules
- `docs/review/gaps-and-issues-r49.md`
- `crates/kamn-core/tests/review_r49_docs_contract.rs`
- `specs/milestones/r50-9-r49-review-artifact-consistency-reconciliation/index.md`
- `specs/5487/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: future branch-count drift makes literal status text stale again.
  - Mitigation: tie status text to explicit post-publication marker context and enforce via docs-contract test.

## Interfaces / Contracts
- Documentation artifact contract only.

## Validation Strategy
- `TMPDIR=/home/n/Code/kamn-r50/.tmp cargo test -p kamn-core --test review_r49_docs_contract`
- `TMPDIR=/home/n/Code/kamn-r50/.tmp cargo fmt --check`
- `TMPDIR=/home/n/Code/kamn-r50/.tmp cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
