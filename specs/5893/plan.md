# Plan: Issue #5893 - Branch-Diff Freeze Guard for Existing r51+ Review Docs

## Approach
1. Extend `crates/kamn-core/tests/review_r53_docs_contract.rs` with helper(s) that read name-status entries from `git diff --name-status origin/main...HEAD -- docs/review`.
2. Add a regression test enforcing non-add prohibition for existing frozen review docs (`r51+`).
3. Run targeted docs-contract lane to validate both existing and new assertions.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`

## Risks and Mitigations
- Risk: environments without `origin/main` cause false failures.
  - Mitigation: resolve merge-base fallback to `main` when available and fail with deterministic guidance only if no baseline ref can be resolved.
- Risk: accidental blocking of new release doc creation.
  - Mitigation: explicitly allow `A` status entries.

## Interfaces / Contracts
- No schema changes.
- Branch-diff enforcement uses existing freeze effective-release marker from `review-document-freeze.policy`.

## ADR
- Not required.
