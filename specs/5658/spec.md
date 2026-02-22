# Spec: #5658 Verification Anchor Height Format Contract

- Issue: #5658
- Milestone: R63 E2E Verification Anchor Height Format Contract
- Status: Reviewed
- Priority: P1

## Problem Statement
Verify enforces `_verification.kolme_anchor.block_height` marker presence but not value format. PRD section 8.3 models `block_height` as a numeric value.

## Scope
### In Scope
- Enforce deterministic rejection when `_verification.kolme_anchor.block_height` value is non-numeric.
- Preserve existing marker-presence, finality-value, and hash-format checks.
- Emit deterministic diagnostics for block-height format violations.

### Out of Scope
- Live chain reconciliation of block-height existence.
- Cross-checking block height with external finality sources.

## Acceptance Criteria
### AC-1 Invalid block-height format rejection
Given `_verification.kolme_anchor.block_height` is present but non-numeric,
When verify command runs,
Then verification fails with deterministic block-height format error.

### AC-2 Deterministic diagnostics
Given invalid block-height format appears in an evidence artifact,
When verify command runs,
Then the error identifies `_verification.kolme_anchor.block_height` format contract violation.

### AC-3 Valid block-height compatibility
Given `_verification.kolme_anchor.block_height` is numeric and other required contracts hold,
When verify command runs,
Then verification report generation succeeds.

## Conformance Cases
- C-01 (AC-1, AC-2): verify rejects non-numeric `block_height` values.
- C-02 (AC-3): verify accepts numeric `block_height` values with other required marker contracts.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with block-height format conformance coverage.
- `cargo test -p kamn-e2e-harness` green with no regressions.
