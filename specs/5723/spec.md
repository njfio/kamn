# Spec: #5723 Execute R52 Spec-Volume Remediation Tranche-4 (14-Dir Reduction)

- Issue: #5723
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
R52 tranche-3 reduced top-level `specs/` count to 810, but spec-volume guardrail pressure remains
high. A fourth deterministic tranche is required to continue downward reduction while preserving
archive-policy integrity and fail-closed docs-contract validation.

## Scope
### In Scope
- Remove 14 additional archived issue-spec pairs by deleting:
  - `specs/<issue-id>/ARCHIVED.md` pointer directories
  - `specs/archive/<issue-id>/` payload directories
  - matching rows in `specs/archive/index.md`
- Update tranche markers in `docs/review/gaps-and-issues-r52.md`.
- Update non-regression ratchet markers in `docs/review/gaps-and-issues-r50.md`.
- Extend existing docs-contract test expectations for tranche-4 values.

### Out of Scope
- Archive-policy model redesign.
- Runtime feature changes.
- CI/workflow topology changes.

## Acceptance Criteria
### AC-1 Deterministic 14-dir tranche reduction
Given tranche-4 pre-count evidence for top-level `specs/` directories,
When tranche-4 cleanup executes,
Then post-count equals `pre - 14`.

### AC-2 Archive policy integrity retained
Given updated archive index and retained archive directories,
When archive policy checker runs,
Then index/pointer/payload counts align and validation passes.

### AC-3 R50 non-regression ratchet refresh
Given the reduced post-tranche top-level `specs/` count,
When R50 non-regression markers are updated,
Then baseline/max values match reduced cap and remain internally consistent.

### AC-4 R52 tranche marker contract coverage
Given updated R52 post-publication tranche markers,
When docs-contract tests parse markers,
Then expected tranche-4 values exist and satisfy `pre - deleted = post`.

## Conformance Cases
- C-01 (AC-1): deterministic marker evidence in `docs/review/gaps-and-issues-r52.md`.
- C-02 (AC-2): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-03 (AC-3, AC-4): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-04 (AC-4): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-05 (AC-3, AC-4): `cargo fmt --all --check`.
- C-06 (AC-3, AC-4): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- Top-level `specs/` directory count decreases by 14 in tranche-4.
- Archive policy checker remains `status=ok` with aligned counts.
- Docs-contract tests fail closed if marker values drift.
