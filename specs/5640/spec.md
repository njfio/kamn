# Spec: #5640 Evidence `_verification` Block Enforcement

- Issue: #5640
- Milestone: R57 E2E Evidence Verification Block Enforcement
- Status: Reviewed
- Priority: P1

## Problem Statement
PRD section 8.3 requires evidence JSON artifacts to include a `_verification` block with deterministic metadata. Current `execute_verify_contract` validates manifest markers only and does not enforce artifact-level `_verification` fields.

## Scope
### In Scope
- Validate `_verification` block marker presence for evidence JSON files in the evidence directory (excluding manifest/support files).
- Emit deterministic missing-marker errors with file path context.
- Preserve existing verify-report output shape for valid evidence.

### Out of Scope
- Hash recomputation and cryptographic validation.
- JSON schema/dependency introduction.

## Acceptance Criteria
### AC-1 Missing `_verification` block rejection
Given an evidence JSON artifact lacking `_verification`,
When verify command runs,
Then verification fails with deterministic marker/path error.

### AC-2 Required `_verification` marker rejection
Given an evidence JSON artifact with `_verification` missing any required marker,
When verify command runs,
Then verification fails with deterministic missing-marker/path error.

### AC-3 Valid `_verification` block compatibility
Given evidence JSON artifacts containing required `_verification` markers,
When verify command runs,
Then deterministic verification report generation succeeds.

### AC-4 Contract stability
Given current verify report contract for valid manifests,
When `_verification` artifact checks are added,
Then report shape (`schema_check`, `proof_check`, `chain_check`, `content_check`) remains unchanged.

## Conformance Cases
- C-01 (AC-1): verify rejects evidence file missing `_verification` block.
- C-02 (AC-2): verify rejects evidence file missing `_verification.kolme_anchor.tx_hash` marker.
- C-03 (AC-3): verify succeeds when evidence file includes complete `_verification` markers.
- C-04 (AC-4): successful verify output still includes deterministic report keys.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with new `_verification` conformance checks.
- `cargo test -p kamn-e2e-harness` remains green.
