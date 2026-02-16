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
  - `rehearsal_boundary_reason_taxonomy_status=verified`
  - `rehearsal_boundary_reason_taxonomy_version=kamn.release.staging-rehearsal-boundary-reason-taxonomy.v1`
  - `rehearsal_boundary_reason_codes_csv=rehearsal_boundary_ci_smoke_seconds_exceeded,rehearsal_boundary_local_heavy_opt_in_missing,rehearsal_runbook_contract_parity_mismatch`
  - `rehearsal_boundary_ci_smoke_max_seconds=120`
  - `rehearsal_boundary_local_heavy_max_seconds=900`
  - local-heavy run path requires explicit opt-in:
    - `KAMN_STAGING_REHEARSAL_LOCAL_HEAVY_OPT_IN=1`
- Reason-surface drift is fail-closed:
  - `reason taxonomy mismatch`
- Normalized-output drift is fail-closed:
  - `normalized evidence bundle mismatch`
- Boundary governance drift is fail-closed:
  - `rehearsal_boundary_ci_smoke_seconds_exceeded`
  - `rehearsal_boundary_local_heavy_opt_in_missing`
  - `rehearsal_runbook_contract_parity_mismatch`

## Go/No-Go Incident Readiness Bundle Convergence Gate (Issue #4470)

Release go/no-go approval can ingest incident-readiness rehearsal evidence and fail closed when
schema/taxonomy/normalization markers drift.

- Incident readiness source bundle generation:
  - `bash scripts/deploy/generate_staging_rehearsal_bundle.sh --output-file /tmp/staging-rehearsal.json --release-candidate v1.1.0-rc.1 --deploy-status PASS --rollback-status PASS --rollback-target-hash state-hash-expected --post-rollback-hash state-hash-expected --recovery-time-seconds 420 --max-allowed-recovery-time-seconds 900 --evidence-complete true --ci-fast-gate PASS`
- Go/no-go bundle generation with incident-readiness convergence gate:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-incident-readiness.json --release-candidate v1.1.0-rc.1 --schema-target-version 1.0.0 --runtime-image-digest sha256:incident-ready --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2 --incident-readiness-report-file /tmp/staging-rehearsal.json --incident-readiness-max-age-seconds 1800`
- Policy check:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-incident-readiness.json`

Deterministic gate markers:
- `incident_readiness_gate_final_decision=GO|NO-GO`
- `reason_taxonomy_version=kamn.release.gonogo-incident-readiness-convergence-reason-taxonomy.v1`
- `reason_codes_csv=none|<csv>`

Mismatch and tamper failure cases:
- `gonogo_incident_readiness_file_missing`
- `gonogo_incident_readiness_invalid_json`
- `gonogo_incident_readiness_freshness_window_exceeded`
- `gonogo_incident_readiness_schema_mismatch`
- `gonogo_incident_readiness_final_decision_not_go`
- `gonogo_incident_readiness_output_contract_version_mismatch`
- `gonogo_incident_readiness_reason_taxonomy_schema_mismatch`
- `gonogo_incident_readiness_normalized_evidence_schema_mismatch`
- `gonogo_incident_readiness_staged_signoff_schema_mismatch`
- `gonogo_incident_readiness_staged_signoff_status_not_verified`
- `gonogo_incident_readiness_reason_codes_unexpected`
- checker drift guard:
  - `incident readiness gate convergence mismatch`

## Regression Markers

- Rehearsal command and output drift coverage:
  - `Regression: #4499`
- Rehearsal reason taxonomy and normalized evidence drift coverage:
  - `Regression: #4500`
- Rehearsal runbook parity and boundary threshold drift coverage:
  - `Regression: #4501`
- Rehearsal boundary reason taxonomy and local-heavy opt-in enforcement drift coverage:
  - `Regression: #4502`
- Incident-readiness bundle mismatch/tamper/stale fail-closed coverage:
  - `Regression: #4469`
