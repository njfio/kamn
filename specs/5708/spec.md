# Spec: #5708 Reconcile R52 Branch Hygiene Drift with Merged-Only Cleanup and Docs-Contract Markers

- Issue: #5708
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
R52 reports branch hygiene drift (`67` remote heads, +6 vs R51) and recommends merged-branch pruning. Without deterministic reconciliation markers and contract coverage, post-cleanup state can drift silently and regress in later cycles.

## Scope
### In Scope
- Measure remote branch baseline before cleanup.
- Identify merged-only remote branch candidates relative to `origin/main`.
- Delete only merged remote branches selected by deterministic filter policy.
- Update `docs/review/gaps-and-issues-r52.md` with branch-hygiene reconciliation markers (`pre`, `deleted`, `post`, command evidence).
- Add a fail-closed docs-contract test asserting marker presence and arithmetic consistency.

### Out of Scope
- Deleting unmerged remote branches.
- Rewriting historical R52 baseline snapshot values.
- Workflow/CI topology changes.

## Acceptance Criteria
### AC-1 Merged-only deletion safety
Given remote branches on `origin`,
When the cleanup wave runs,
Then every deleted branch is already merged into `origin/main`.

### AC-2 Deterministic reconciliation markers
Given `docs/review/gaps-and-issues-r52.md`,
When branch-hygiene markers are parsed,
Then `pre`, `deleted`, and `post` markers exist and satisfy `pre - deleted = post`.

### AC-3 Fail-closed docs contract
Given marker drift/removal in the R52 branch-hygiene reconciliation section,
When docs-contract tests run,
Then the R52 branch-hygiene docs-contract suite fails deterministically.

### AC-4 Branch count improvement signal
Given cleanup execution,
When post-cleanup inventory is measured,
Then post-cleanup remote branch count is lower than or equal to R52 baseline snapshot.

## Conformance Cases
- C-01 (AC-1, AC-4): merged-only branch cleanup evidence command log.
- C-02 (AC-2, AC-3): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-03 (AC-2, AC-3): `cargo fmt --all --check`.
- C-04 (AC-2, AC-3): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- R52 branch count reconciliation is documented with deterministic markers.
- Docs-contract lane prevents silent marker drift.
- Branch count trend is non-increasing versus the R52 baseline snapshot.
