# Plan — Issue #4830

## Approach

1. Add a standalone drift policy checker wrapper around `generate_lane_artifacts.py --mode check`.
2. Emit deterministic fail-closed reason taxonomy outputs from the checker for both pass/fail paths.
3. Add a shell contract test that:
   - validates GO markers against current repository state
   - validates NO-GO markers against a tampered mini-repo manifest fixture
4. Wire drift checker test into framework regression runner and validate with full CI tools suite.

## Affected Modules

- `scripts/framework/check_lane_registry_drift.sh`
- `scripts/framework/test_check_lane_registry_drift.sh`
- `scripts/framework/test_contract_framework.sh`
- `docs/architecture/lane-registry-generation.md`

## Risks / Mitigations

- Risk: false positives from drift checker due to environment path assumptions.
  Mitigation: checker accepts explicit `--repo-root` and `--registry-file` overrides.
- Risk: unstable fail messages reduce governance usefulness.
  Mitigation: deterministic reason taxonomy and explicit reason-code mapping by failure class.
- Risk: CI runtime growth.
  Mitigation: drift check added under existing framework test entrypoint and validated in existing CI suite.

## Interfaces / Contracts

- Checker taxonomy version:
  - `kamn.framework.lane-registry-drift-reason-taxonomy.v1`
- Checker output contract:
  - `status=ok|fail`
  - `final_decision=GO|NO-GO`
  - `reason_taxonomy_version=<...>`
  - `reason_codes=<...>`
- Failure reason mapping:
  - `lane_registry_manifest_drift_detected`
  - `lane_registry_wrapper_drift_detected`
  - `lane_registry_schema_mismatch`
  - `lane_registry_artifact_missing`
  - fallback `lane_registry_check_failed`

## ADR

No ADR required for this subtask. No dependency/protocol boundary changes were introduced.
