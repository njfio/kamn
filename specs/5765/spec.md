# Spec: #5765 Add R52 Governance-Feature 70/30 Target Reconciliation Contract

- Issue: #5765
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
`docs/review/gaps-and-issues-r52.md` captures snapshot governance-vs-feature ratios and includes a
recommendation to budget the next cycle at least 70/30 feature-vs-governance, but no fail-closed
marker contract records that target with deterministic status semantics.

## Scope
### In Scope
- Add post-publication governance-feature target reconciliation contract guidance to
  `docs/review/README.md`.
- Add additive marker block to `docs/review/gaps-and-issues-r52.md` recording:
  - snapshot governance/feature ratios,
  - explicit 70/30 target bounds,
  - reconciliation status,
  - snapshot-preservation marker.
- Extend docs-contract tests to enforce marker presence and cross-marker consistency.
- Add lifecycle artifacts and milestone slice tracking.
- Perform compensating single archived issue-spec pair cleanup to preserve top-level `specs/`
  non-regression cap (`<= 693`) after adding `specs/5765`.

### Out of Scope
- Rewriting historical R52 snapshot narrative rows.
- Commit-history recomputation or reclassification.
- CI/workflow changes.

## Acceptance Criteria
### AC-1 Contract documented
Given the R52 70/30 recommendation,
When review marker contracts are updated,
Then README defines required reconciliation keys and invariants.

### AC-2 R52 markers published
Given snapshot ratios already recorded in Section 5.3,
When additive reconciliation markers are added,
Then R52 contains target-bound markers and status fields for governance-feature budgeting.

### AC-3 Cross-marker consistency
Given snapshot and reconciliation markers,
When docs-contract tests parse marker values,
Then snapshot markers match existing ratio markers, target bounds represent 70/30, and status is
consistent with recorded snapshot ratios.

### AC-4 Snapshot preservation
Given historical review narrative content,
When reconciliation markers are appended,
Then existing snapshot rows remain unchanged.

### AC-5 Fail-closed enforcement
Given required contract markers,
When keys drift or values become inconsistent,
Then docs-contract tests fail.

### AC-6 Non-regression cap preservation
Given one new lifecycle spec directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): README includes governance-feature target reconciliation contract keys/invariants.
- C-02 (AC-2): R52 doc contains required reconciliation markers.
- C-03 (AC-3/AC-5): docs-contract test validates cross-marker equality and 70/30 bounds.
- C-04 (AC-4): baseline Section 5.2 recommendation text remains unchanged.
- C-05 (AC-6): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-06 (AC-6): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-07 (AC-1..AC-6): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-08 (AC-1..AC-6): `cargo fmt --all --check`.
- C-09 (AC-1..AC-6): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- New governance-feature reconciliation marker block exists and is test-enforced.
- 70/30 target bound is machine-readable and deterministic.
- `specs/` cap remains non-regressing (`<= 693`).
