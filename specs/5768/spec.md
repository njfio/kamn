# Spec: #5768 Add R52 Feat-Labeling Post-Publication Reconciliation Contract

- Issue: #5768
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
R52 records feat mislabeling as an open priority row (`4 of 15`) but lacks deterministic marker
contracts that encode snapshot counts/ratio and post-publication status semantics for downstream
fail-closed governance checks.

## Scope
### In Scope
- Add post-publication feat-labeling reconciliation contract guidance in `docs/review/README.md`.
- Add additive marker block in `docs/review/gaps-and-issues-r52.md` with snapshot counts, ratio,
  recommended prefix surface, and status markers.
- Extend docs-contract tests in
  `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs` for presence
  and consistency checks.
- Add lifecycle artifacts and milestone tracking.
- Perform compensating single archived issue-spec pair cleanup to preserve top-level `specs/`
  non-regression cap (`<= 693`) after adding `specs/5768`.

### Out of Scope
- Retrospective commit reclassification.
- CI/workflow changes.
- Automated commit-label enforcement.

## Acceptance Criteria
### AC-1 Contract documented
Given R52 feat-mislabeling is an open priority,
When review contract docs are updated,
Then README includes required feat-labeling reconciliation marker keys and invariants.

### AC-2 R52 markers published
Given R52 snapshot counts are known,
When additive marker block is added,
Then R52 includes deterministic snapshot count/ratio/status markers.

### AC-3 Consistency enforcement
Given snapshot count and ratio markers,
When docs-contract tests parse them,
Then ratio equals count/total and status/preservation markers remain deterministic.

### AC-4 Snapshot preservation
Given historical priority rows,
When reconciliation markers are appended,
Then the baseline feat-mislabeling row text remains unchanged.

### AC-5 Fail-closed checks
Given marker drift or removal,
When docs-contract tests run,
Then failures occur deterministically.

### AC-6 Non-regression cap preservation
Given one new lifecycle spec directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): README includes feat-labeling reconciliation contract keys/invariants.
- C-02 (AC-2): R52 doc includes feat-labeling reconciliation marker block.
- C-03 (AC-3/AC-5): docs-contract tests validate count/ratio consistency and required status markers.
- C-04 (AC-4): historical feat-mislabeling priority row text remains unchanged.
- C-05 (AC-6): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-06 (AC-6): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-07 (AC-1..AC-6): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-08 (AC-1..AC-6): `cargo fmt --all --check`.
- C-09 (AC-1..AC-6): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- Feat-labeling reconciliation markers are present and enforced by docs-contract tests.
- Snapshot ratio marker is consistent with snapshot counts.
- Top-level `specs/` cap remains non-regressing (`<= 693`).
