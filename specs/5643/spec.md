# Spec: #5643 Chain Dump Marker Validation

- Issue: #5643
- Milestone: R58 E2E Chain Dump Verification Contract Hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
Verify flow currently checks only existence of `--kolme-chain-dump` path. Chain validation semantics should enforce required chain dump content markers to make `chain_check` meaningful.

## Scope
### In Scope
- Enforce required chain dump markers in verify path: `chain_name`, `chain_version`, `blocks`.
- Emit deterministic missing-marker errors.
- Preserve verify report output shape for valid inputs.

### Out of Scope
- Chain structure/schema validation.
- Cross-checking block values against evidence content.

## Acceptance Criteria
### AC-1 Missing chain marker rejection
Given chain dump content missing a required marker,
When verify command runs,
Then verification fails with deterministic missing-marker error.

### AC-2 Valid chain marker compatibility
Given chain dump content containing required markers,
When verify command runs,
Then verification report generation succeeds.

### AC-3 Contract stability
Given existing verify report contract,
When chain marker checks are added,
Then report output keys remain unchanged (`schema_check`, `proof_check`, `chain_check`, `content_check`).

## Conformance Cases
- C-01 (AC-1): verify rejects chain dump missing `chain_name`.
- C-02 (AC-1): verify rejects chain dump missing `blocks`.
- C-03 (AC-2): verify accepts chain dump with required markers.
- C-04 (AC-3): successful verify report still includes deterministic check markers.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with chain-dump marker assertions.
- `cargo test -p kamn-e2e-harness` remains green.
