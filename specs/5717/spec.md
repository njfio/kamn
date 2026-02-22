# Spec: #5717 Execute R52 Spec-Volume Remediation Tranche-2 (14-Dir Reduction)

- Issue: #5717
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
R52 tranche-1 reduced top-level `specs/` directory count, but guardrail pressure remains elevated.
A second deterministic tranche is required to continue downward movement while preserving archive
index integrity and fail-closed review marker contracts.

## Scope
### In Scope
- Remove 14 additional low-value archived issue-spec pairs by deleting:
  - `specs/<issue-id>/ARCHIVED.md` pointer directories
  - `specs/archive/<issue-id>/` payload directories
  - matching rows in `specs/archive/index.md`
- Record tranche-2 pre/delete/post markers in `docs/review/gaps-and-issues-r52.md`.
- Extend existing spec-volume docs-contract tests to validate tranche-2 marker presence and
  arithmetic invariants.
- Refresh `docs/review/gaps-and-issues-r50.md` non-regression marker baseline/max to the reduced
  count resulting from tranche-2.

### Out of Scope
- Archive policy redesign.
- Runtime feature changes.
- CI/workflow topology changes.

## Acceptance Criteria
### AC-1 Deterministic 14-dir reduction
Given tranche-2 pre-count evidence for top-level `specs/` directories,
When tranche-2 cleanup executes,
Then post-count equals `pre - 14`.

### AC-2 Archive integrity for retained corpus
Given updated `specs/archive/index.md` and retained archive directories,
When archive policy checker runs,
Then validation succeeds with no orphaned pointers/payloads.

### AC-3 R50 non-regression ratchet refresh
Given post-tranche reduced `specs/` directory count,
When R50 non-regression markers are updated,
Then baseline/max markers match the reduced cap and remain internally consistent.

### AC-4 R52 tranche-2 markers enforced by docs-contract tests
Given R52 post-publication tranche marker section,
When docs-contract tests parse markers,
Then tranche-2 marker keys exist and satisfy `pre - deleted = post` with deleted count fixed at 14.

## Conformance Cases
- C-01 (AC-1): deterministic count evidence markers in `docs/review/gaps-and-issues-r52.md`.
- C-02 (AC-2): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>` returns `status=ok`.
- C-03 (AC-3, AC-4): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-04 (AC-4): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-05 (AC-3, AC-4): `cargo fmt --all --check`.
- C-06 (AC-3, AC-4): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- Net top-level `specs/` directory count decreases by 14 in tranche-2.
- Archive policy checker remains green for retained archive set.
- Docs-contract tests fail closed on tranche-2 marker regressions.
