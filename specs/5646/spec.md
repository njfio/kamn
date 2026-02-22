# Spec: #5646 Chain Dump Hash Continuity Verification

- Issue: #5646
- Milestone: R59 E2E Chain Hash Continuity Verification Contract
- Status: Implemented
- Priority: P1

## Problem Statement
Verify flow currently enforces chain-dump marker presence only (`chain_name`, `chain_version`, `blocks`). PRD section 9 requires chain hash continuity verification from genesis; malformed chains can currently pass verification.

## Scope
### In Scope
- Enforce deterministic continuity validation for chain-dump block hashes in verify flow.
- Enforce required per-block hash markers for continuity checks.
- Emit deterministic continuity mismatch diagnostics.

### Out of Scope
- Cryptographic recomputation of Kolme block hashes.
- Cross-checking chain-dump blocks against live network state.

## Acceptance Criteria
### AC-1 Missing block hash marker rejection
Given a chain dump block missing required hash continuity markers,
When verify command runs,
Then verification fails with deterministic missing-marker error.

### AC-2 Continuity mismatch rejection
Given a chain dump with ordered blocks whose `previous_block_hash` does not match the prior block `block_hash`,
When verify command runs,
Then verification fails with deterministic continuity-mismatch error.

### AC-3 Valid continuity compatibility
Given a chain dump with required markers and coherent continuity from genesis,
When verify command runs,
Then verification report generation succeeds.

## Conformance Cases
- C-01 (AC-1): verify rejects chain dump where a block is missing `block_hash`.
- C-02 (AC-1): verify rejects chain dump where a block is missing `previous_block_hash`.
- C-03 (AC-2): verify rejects chain dump with continuity mismatch at block index `n`.
- C-04 (AC-3): verify accepts chain dump with coherent block hash continuity.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with chain continuity assertions.
- `cargo test -p kamn-e2e-harness` remains green.
