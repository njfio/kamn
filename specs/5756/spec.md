# Spec: #5756 Reconcile R52 Post-Publication Branch-Hygiene Status Markers

- Issue: #5756
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
R52 includes deterministic post-publication branch cleanup count markers, but branch-hygiene
status language remains historical baseline text (`SLIGHTLY WORSENED`) without additive
post-publication status markers. We need explicit marker reconciliation that preserves baseline rows
while making current branch posture machine-verifiable.

## Scope
### In Scope
- Add post-publication branch-hygiene status reconciliation marker contract guidance to
  `docs/review/README.md`.
- Add additive status reconciliation marker block to `docs/review/gaps-and-issues-r52.md`.
- Extend docs-contract tests for marker presence and consistency against existing branch cleanup
  markers.
- Perform compensating single archived issue-spec pair cleanup for issue `3873`
  (`specs/3873/ARCHIVED.md`, `specs/archive/3873/`, `specs/archive/index.md`) to preserve the
  `<= 693` spec-dir non-regression cap after adding `specs/5756`.

### Out of Scope
- Rewriting historical baseline branch rows.
- Runtime behavior changes.
- CI/workflow topology changes.

## Acceptance Criteria
### AC-1 Branch-hygiene status reconciliation markers published
Given baseline branch sections remain unchanged,
When post-publication status reconciliation is recorded,
Then R52 includes additive markers describing snapshot status and reconciled post-publication
status.

### AC-2 Marker consistency
Given status reconciliation markers and existing branch cleanup markers,
When docs-contract tests parse marker values,
Then branch status markers are internally consistent with cleanup counts and snapshot baseline.

### AC-3 Snapshot preservation
Given R52 snapshot semantics,
When additive status markers are added,
Then baseline section text and priority row text remain unchanged.

### AC-4 Fail-closed docs-contract enforcement
Given marker keys are required,
When markers drift or are removed,
Then docs-contract tests fail.

### AC-5 Non-regression cap preservation
Given one new lifecycle spec directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): R52 docs contain status reconciliation marker block.
- C-02 (AC-2/AC-4): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-03 (AC-3): baseline branch heading/priority row lines remain unchanged.
- C-04 (AC-5): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-05 (AC-5): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-06 (AC-2/AC-4/AC-5): `cargo fmt --all --check`.
- C-07 (AC-2/AC-4/AC-5): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- Branch-hygiene status reconciliation markers are present and validated by docs-contract tests.
- Existing branch cleanup counts and status reconciliation markers are consistent.
- Top-level `specs/` directory count remains cap-compliant (`<= 693`).
