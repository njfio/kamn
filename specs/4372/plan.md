# Plan — #4372

## Approach
- Add RED lineage mismatch tests where evidence paths remain present but violate submit/finality linkage invariants.
- Extend policy checker with explicit artifact lineage mismatch checks and deterministic provider-failure taxonomy output fields.
- Update contract-lane harness and documentation marker coverage to lock behavior.

## Affected Modules
- `scripts/kolme/test_check_local_runtime_commit_live_evidence_policy.sh`
- `scripts/kolme/test_run_local_runtime_commit_live_finality_evidence_contract_lane.sh`
- `scripts/kolme/check_local_runtime_commit_live_evidence_policy.py`
- `scripts/kolme/contracts/local_runtime_commit_live_finality_evidence_contract_lane.py`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations
- Risk: taxonomy overgrowth causing brittle checks.
  - Mitigation: define a bounded provider-failure taxonomy set and normalize deterministic ordering.
- Risk: overlap with existing reason codes.
  - Mitigation: introduce precise lineage mismatch reason names without changing established submit/finality mismatch markers.

## Interfaces/Contracts
- New lineage drift reasons:
  - `request_payload_evidence_artifact_path_lineage_mismatch`
  - `submit_evidence_artifact_path_lineage_mismatch`
  - `finality_evidence_artifact_path_lineage_mismatch`
- New policy output fields:
  - `provider_failure_reason_taxonomy_version`
  - `provider_failure_reason_codes_csv`
  - `provider_failure_reason_codes_value`

## ADR
- Not required (no architecture/dependency/protocol change).
