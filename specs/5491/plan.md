# Issue #5491 Plan - R49 Branch-Count Reconciliation Execution

## Approach
1. Update stale `52` values in post-publication R49 review artifact sections to `51`.
2. Update docs-contract test expectations to match reconciled markers/highlight text.
3. Validate via targeted docs-contract test and formatting checks.

## Affected Modules
- `docs/review/gaps-and-issues-r49.md`
- `crates/kamn-core/tests/review_r49_docs_contract.rs`
- `specs/milestones/r50-11-r49-post-publication-branch-count-reconciliation/index.md`
- `specs/5491/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: updating only one section and leaving marker drift.
  - Mitigation: update all post-publication sections + marker + docs-contract assertions.

## Interfaces / Contracts
- Documentation contract only; no runtime interfaces changed.

## Validation Strategy
- `cargo test -p kamn-core --test review_r49_docs_contract`
- `cargo fmt --check`
