# Incident Readiness Runbook

This runbook defines local rehearsal command contracts and drift guards for staged release evidence.

## Staging Rehearsal Commands

- Bundle generation:
  - `bash scripts/deploy/generate_staging_rehearsal_bundle.sh --output-file /tmp/staging-rehearsal.json --release-candidate v1.1.0-rc.1 --deploy-status PASS --rollback-status PASS --rollback-target-hash state-hash-expected --post-rollback-hash state-hash-expected --recovery-time-seconds 420 --max-allowed-recovery-time-seconds 900 --evidence-complete true --ci-fast-gate PASS`
- Policy check:
  - `bash scripts/deploy/check_staging_rehearsal_policy.sh --bundle-file /tmp/staging-rehearsal.json`
- Contract lane:
  - `bash scripts/deploy/run_staging_rehearsal_contract_lane.sh`
- Deep rehearsal lane:
  - `bash scripts/deploy/run_staging_rehearsal_deep_lane.sh`

## Drift Guards

- Command-surface drift is fail-closed:
  - `command contract mismatch`
- Evidence-output contract drift is fail-closed:
  - `evidence output contract version mismatch`
- Deterministic taxonomy and normalization surfaces:
  - `rehearsal_reason_taxonomy_version=kamn.release.staging-rehearsal-reason-taxonomy.v1`
  - `rehearsal_normalized_evidence_version=kamn.release.staging-rehearsal-evidence-normalization.v1`
- Runbook/boundary governance surfaces:
  - `rehearsal_runbook_contract_parity_status=verified`
  - `rehearsal_boundary_thresholds_schema_version=kamn.release.staging-rehearsal-boundary-thresholds.v1`
  - `rehearsal_boundary_ci_smoke_max_seconds=120`
  - `rehearsal_boundary_local_heavy_max_seconds=900`
- Reason-surface drift is fail-closed:
  - `reason taxonomy mismatch`
- Normalized-output drift is fail-closed:
  - `normalized evidence bundle mismatch`
- Boundary governance drift is fail-closed:
  - `rehearsal_boundary_ci_smoke_seconds_exceeded`
  - `rehearsal_boundary_local_heavy_opt_in_missing`
  - `rehearsal_runbook_contract_parity_mismatch`

## Regression Markers

- Rehearsal command and output drift coverage:
  - `Regression: #4499`
- Rehearsal reason taxonomy and normalized evidence drift coverage:
  - `Regression: #4500`
- Rehearsal runbook parity and boundary threshold drift coverage:
  - `Regression: #4501`
