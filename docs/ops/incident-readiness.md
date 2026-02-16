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
- Reason-surface drift is fail-closed:
  - `reason taxonomy mismatch`
- Normalized-output drift is fail-closed:
  - `normalized evidence bundle mismatch`

## Regression Markers

- Rehearsal command and output drift coverage:
  - `Regression: #4499`
- Rehearsal reason taxonomy and normalized evidence drift coverage:
  - `Regression: #4500`
