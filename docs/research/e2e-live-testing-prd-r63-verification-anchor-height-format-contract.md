# E2E Live Testing PRD R63 Verification Anchor Height Format Contract

## Context
This artifact records R63 hardening for PRD section 8.3 anchor-marker semantics requiring `_verification.kolme_anchor.block_height` to be numeric.

## Baseline (Before #5658)
- `r63_verification_anchor_height_format_contract_status_before=missing`
- `r63_verify_anchor_block_height_format_enforcement=missing`

## Implemented in #5658
- Added deterministic verify rejection when `_verification.kolme_anchor.block_height` is non-numeric.
- Verify now fails with deterministic diagnostic:
  - `evidence artifact invalid _verification.kolme_anchor.block_height format: <artifact-path>`
- Existing marker-presence, finality-value, and hash-format checks remain enforced.

## Status Markers (After #5658)
- `r63_verify_anchor_block_height_format_enforcement=implemented`
- `r63_verification_anchor_height_format_contract_status_after=implemented`
