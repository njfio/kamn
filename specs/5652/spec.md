# Spec: #5652 Verification Anchor Finality Value Contract

- Issue: #5652
- Milestone: R61 E2E Verification Finality Value Contract
- Status: Implemented
- Priority: P1

## Problem Statement
Verify currently enforces `_verification.kolme_anchor.finality` marker presence but does not enforce the PRD-required value `FINAL`, allowing invalid finality states to pass contract checks.

## Scope
### In Scope
- Enforce deterministic rejection when `_verification.kolme_anchor.finality` is present but not `FINAL`.
- Preserve existing marker-presence checks for `_verification` and `kolme_anchor` fields.
- Emit deterministic diagnostics for invalid finality values.

### Out of Scope
- RPC-based finality verification against live chain state.
- Support for alternate finality states beyond PRD contract.

## Acceptance Criteria
### AC-1 Invalid finality value rejection
Given evidence artifact `_verification.kolme_anchor.finality` has a non-`FINAL` value,
When verify command runs,
Then verification fails with deterministic invalid-finality error.

### AC-2 Deterministic value-specific diagnostic
Given invalid finality value appears in an evidence artifact,
When verify command runs,
Then the error identifies `_verification.kolme_anchor.finality` contract violation.

### AC-3 Valid finality compatibility
Given evidence artifacts contain `_verification.kolme_anchor.finality` with value `FINAL`,
When verify command runs,
Then verification report generation succeeds.

## Conformance Cases
- C-01 (AC-1, AC-2): verify rejects evidence artifact with `_verification.kolme_anchor.finality` set to non-`FINAL` value.
- C-02 (AC-3): verify accepts evidence artifact with `_verification.kolme_anchor.finality` set to `FINAL`.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with finality-value conformance coverage.
- `cargo test -p kamn-e2e-harness` green with no regressions.
