# E2E Live Testing PRD R62 Verification Hash Format Contract

## Context
This artifact records R62 hardening for PRD section 8.3 hash-marker semantics requiring `sha256:` value format for evidence verification hashes.

## Baseline (Before #5655)
- `r62_verification_hash_format_contract_status_before=missing`
- `r62_verify_artifact_hash_format_enforcement=missing`

## Implemented in #5655
- Added deterministic verify rejection when `_verification.evidence_hash` is not a non-empty `sha256:` value.
- Added deterministic verify rejection when `_verification.kolme_anchor.tx_hash` is not a non-empty `sha256:` value.
- Verify now fails with deterministic diagnostics:
  - `evidence artifact invalid _verification.evidence_hash format: <artifact-path>`
  - `evidence artifact invalid _verification.kolme_anchor.tx_hash format: <artifact-path>`
- Existing marker-presence and finality-value checks remain enforced.

## Status Markers (After #5655)
- `r62_verify_artifact_hash_format_enforcement=implemented`
- `r62_verification_hash_format_contract_status_after=implemented`
