# E2E Live Testing PRD R57 Evidence `_verification` Block Enforcement

## Context
This artifact records R57 enforcement of PRD section 8.3 evidence artifact `_verification` marker contracts in verify command flows.

## Baseline (Before #5640)
- `r57_evidence_verification_block_contract_status_before=missing`
- `r57_verify_artifact_verification_marker_enforcement=missing`

## Implemented in #5640
- Added deterministic recursive evidence artifact scanner for `.json` evidence files.
- Verify flow now validates `_verification` block presence for evidence artifacts (excluding support files).
- Verify flow enforces required `_verification` markers:
  - `evidence_hash`
  - `captured_at`
  - `source_node`
  - `agent`
  - `kolme_anchor`
  - `kolme_anchor.tx_hash`
  - `kolme_anchor.block_height`
  - `kolme_anchor.finality`
- Missing marker failures now include deterministic marker path + artifact path context.

## Status Markers (After #5640)
- `r57_verify_artifact_verification_marker_enforcement=implemented`
- `r57_evidence_verification_block_contract_status_after=implemented`
