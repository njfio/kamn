# Plan — #4377

## Approach
- Extend checker shell tests with cross-linked artifact-path drift fixtures and expected mismatch reasons.
- Extend finality contract-lane shell tests with stale finality lineage drift fixture and expected mismatch reason.

## Risks
- Risk: duplicate drift tests overlap with existing missing-path tests.
  - Mitigation: use mismatch-but-present artifact paths to target lineage-specific failures.

## Interfaces
- Expected reasons:
  - `request_payload_evidence_artifact_path_lineage_mismatch`
  - `submit_evidence_artifact_path_lineage_mismatch`
  - `finality_evidence_artifact_path_lineage_mismatch`

## ADR
- Not required.
