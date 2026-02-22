# Spec: #5637 Verify Manifest Nested Field Hardening

- Issue: #5637
- Milestone: R56 E2E Verify Manifest Contract Hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`verify_manifest` validates only top-level markers. PRD section 8.2 defines required nested `infrastructure` and `summary` fields that must be present for deterministic offline verification confidence.

## Scope
### In Scope
- Enforce required `infrastructure` nested markers.
- Enforce required `summary` nested markers.
- Surface deterministic verify-command errors for missing nested markers.

### Out of Scope
- JSON parser dependency introduction.
- Full schema validation implementation.
- Cryptographic proof verification expansion.

## Acceptance Criteria
### AC-1 Infrastructure marker enforcement
Given a manifest missing a required `infrastructure` field,
When verify command runs,
Then verification fails with a deterministic missing-field error marker.

### AC-2 Summary marker enforcement
Given a manifest missing a required `summary` field,
When verify command runs,
Then verification fails with a deterministic missing-field error marker.

### AC-3 Valid manifest compatibility
Given a complete PRD-aligned manifest,
When verify command runs,
Then deterministic verification report generation still succeeds.

### AC-4 Contract stability
Given existing verify report output contract,
When nested-field checks are added,
Then schema/proof/chain/content report markers remain unchanged for valid manifests.

## Conformance Cases
- C-01 (AC-1): verify command rejects manifest missing `infrastructure.kolme_version`.
- C-02 (AC-2): verify command rejects manifest missing `summary.proofs_verified`.
- C-03 (AC-3): existing valid-manifest verify flow remains green.
- C-04 (AC-4): verify report output still contains deterministic check markers.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with new missing-field checks.
- `cargo test -p kamn-e2e-harness` remains green.
