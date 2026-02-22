# Spec: #5655 Verification Hash Format Contract

- Issue: #5655
- Milestone: R62 E2E Verification Hash Format Contract
- Status: Implemented
- Priority: P1

## Problem Statement
Verify enforces `_verification.evidence_hash` and `_verification.kolme_anchor.tx_hash` marker presence, but not required value format. PRD section 8.3 evidence contract examples encode these hashes as `sha256:...`.

## Scope
### In Scope
- Enforce deterministic rejection when `_verification.evidence_hash` is not a non-empty `sha256:` value.
- Enforce deterministic rejection when `_verification.kolme_anchor.tx_hash` is not a non-empty `sha256:` value.
- Preserve existing marker-presence and finality-value checks.

### Out of Scope
- Cryptographic recomputation/verification of hash material.
- Cross-checking hash values against live chain state.

## Acceptance Criteria
### AC-1 Evidence hash format rejection
Given `_verification.evidence_hash` is present but not `sha256:`-prefixed,
When verify command runs,
Then verification fails with deterministic evidence-hash format error.

### AC-2 Anchor tx hash format rejection
Given `_verification.kolme_anchor.tx_hash` is present but not `sha256:`-prefixed,
When verify command runs,
Then verification fails with deterministic anchor-hash format error.

### AC-3 Valid hash format compatibility
Given both hash markers are present with `sha256:` values,
When verify command runs,
Then verification report generation succeeds.

## Conformance Cases
- C-01 (AC-1): verify rejects non-`sha256:` `_verification.evidence_hash` values.
- C-02 (AC-2): verify rejects non-`sha256:` `_verification.kolme_anchor.tx_hash` values.
- C-03 (AC-3): verify accepts valid `sha256:` hash-marker values.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with hash-format conformance coverage.
- `cargo test -p kamn-e2e-harness` green with no regressions.
