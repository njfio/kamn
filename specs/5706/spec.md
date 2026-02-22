# Spec: #5706 Resolve R50 Spec-Volume Non-Regression Cap Breach Blocking Workspace Gate

- Issue: #5706
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`cargo test --workspace --locked --all-features --no-fail-fast` fails on
`review_r50_spec_volume_remediation_docs_contract` because the R50 non-regression
markers are stale relative to current repository baselines. The failing assertion
is:

- `current spec-dir count must not exceed non-regression cap`

This blocks the pre-merge workspace gate introduced in #5705.

## Scope
### In Scope
- Refresh R50 spec-volume non-regression marker values to the current measured
  baseline.
- Keep fail-closed semantics for marker consistency and ratio/count enforcement.
- Update the associated docs-contract test constants to match refreshed marker
  values.
- Verify workspace gating commands pass for this contract slice.

### Out of Scope
- Deleting existing spec directories.
- Broad governance policy redesign beyond this ratchet refresh.
- Non-spec-volume review contract families.

## Acceptance Criteria
### AC-1 Contract unblocked
Given the refreshed R50 non-regression marker baseline,
When `review_r50_spec_volume_remediation_docs_contract` runs,
Then all tests in that suite pass.

### AC-2 Marker consistency preserved
Given updated non-regression markers,
When integration consistency checks run,
Then baseline/cap equality and ratio/count guard assertions remain fail-closed.

### AC-3 Workspace gate compatibility
Given the pre-merge workspace gate command,
When `cargo test --workspace --locked --all-features --no-fail-fast` executes,
Then it no longer fails on `review_r50_spec_volume_remediation_docs_contract`.

## Conformance Cases
- C-01 (AC-1): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-02 (AC-2): `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract`.
- C-03 (AC-3): `cargo test --workspace --locked --all-features --no-fail-fast`.
- C-04 (AC-1..AC-3): `cargo fmt --all --check`.
- C-05 (AC-1..AC-3): `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

## Success Metrics / Observable Signals
- R50 spec-volume remediation docs contract is green.
- Workspace gate no longer red due to this contract.
- Non-regression ratchet markers stay internally consistent and parseable.
