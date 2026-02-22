# Spec: #5714 Execute R52 Spec-Volume Remediation Tranche-1 (14-Dir Reduction)

- Issue: #5714
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
Spec-volume remains severely breached and repeatedly regresses upward. Prior cycles largely ratcheted
non-regression caps without sustained downward movement in top-level `specs/` directory count.

## Scope
### In Scope
- Delete 14 low-value archived issue-spec pairs by removing:
  - `specs/<issue-id>/` pointer directories containing `ARCHIVED.md`
  - `specs/archive/<issue-id>/` archived payload directories
  - matching rows in `specs/archive/index.md`
- Refresh archive index metadata count.
- Refresh R50 spec-volume non-regression markers to reduced baseline.
- Add R52 post-publication tranche-1 spec-volume remediation markers with deterministic pre/delete/post evidence commands.
- Extend existing spec-volume docs-contract tests (same file) to fail closed on new tranche markers.

### Out of Scope
- Archive-policy model redesign.
- Runtime or product behavior changes.
- CI/workflow topology changes.

## Acceptance Criteria
### AC-1 Deterministic 14-dir reduction
Given pre-tranche top-level `specs/` directory count,
When tranche-1 cleanup executes,
Then post-tranche count is exactly `pre - 14`.

### AC-2 Archive policy integrity for retained archive set
Given archive index + pointer policy,
When tranche-1 deletes selected archived pairs and updates index rows/count,
Then archive policy checker passes for remaining archived entries.

### AC-3 Non-regression ratchet refresh
Given R50 spec-volume non-regression ratchet markers,
When tranche-1 lands,
Then baseline/max markers are refreshed to the reduced top-level `specs/` count.

### AC-4 R52 post-publication tranche markers
Given R52 review artifact post-publication reconciliation section,
When markers are parsed,
Then required tranche markers exist and satisfy `pre - deleted = post`.

## Conformance Cases
- C-01 (AC-1): deterministic pre/delete/post `specs/` count evidence.
- C-02 (AC-2): `scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-03 (AC-3, AC-4): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-04 (AC-4): targeted R52 reconciliation docs-contract suite remains green.
- C-05 (AC-3, AC-4): `cargo fmt --all --check`.
- C-06 (AC-3, AC-4): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- Top-level `specs/` directory count drops by 14 in one deterministic tranche.
- Archive policy checker stays green for retained archive corpus.
- Spec-volume non-regression cap moves downward and CI remains green.
