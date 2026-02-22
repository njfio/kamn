# Spec: #5649 Chain Dump Genesis Anchor Verification

- Issue: #5649
- Milestone: R60 E2E Chain Genesis Anchor Verification Contract
- Status: Implemented
- Priority: P1

## Problem Statement
Verify flow checks adjacent block hash linkage but does not assert that chain continuity begins at genesis. PRD section 7.6 and section 9 require verification of hash continuity from genesis.

## Scope
### In Scope
- Enforce deterministic rejection when the first chain-dump block does not anchor to `GENESIS`.
- Preserve existing continuity and marker validations.
- Emit deterministic diagnostics for genesis-anchor contract violations.

### Out of Scope
- Cryptographic reconstruction of block hashes.
- Network RPC reconciliation against live Kolme nodes.

## Acceptance Criteria
### AC-1 Genesis anchor rejection
Given a chain dump whose first block has `previous_block_hash` different from `GENESIS`,
When verify command runs,
Then verification fails with deterministic genesis-anchor mismatch error.

### AC-2 Deterministic index-specific diagnostic
Given the genesis anchor contract is violated at the first block,
When verify command runs,
Then the error includes block index `0` to identify the failed anchor boundary.

### AC-3 Valid genesis continuity compatibility
Given a chain dump where block 0 anchors to `GENESIS` and subsequent blocks maintain continuity,
When verify command runs,
Then verification report generation succeeds.

## Conformance Cases
- C-01 (AC-1, AC-2): verify rejects a chain dump whose first block `previous_block_hash` is not `GENESIS` with deterministic index `0` mismatch diagnostic.
- C-02 (AC-3): verify accepts a chain dump whose first block anchors to `GENESIS` and whose remaining blocks maintain pairwise continuity.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with genesis-anchor conformance coverage.
- `cargo test -p kamn-e2e-harness` green with no regressions in existing verify behavior.
