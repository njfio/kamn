# Tasks: Issue #4469

Status: Completed
Issue: #4469

## Ordered Tasks

T1 (RED):
- Add mismatch/tamper/stale incident-readiness scenarios to
  `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- Capture failing output before implementation.

T2 (GREEN):
- Complete implementation support so new incident-readiness RED tests pass.

T3 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --incident-readiness-report-file ... --incident-readiness-max-age-seconds 1800`

- GREEN command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`

- Regression summary:
  - RED tests now cover incident-readiness taxonomy drift
    (`gonogo_incident_readiness_reason_taxonomy_schema_mismatch`), stale artifact rejection
    (`gonogo_incident_readiness_freshness_window_exceeded`), and tampered gate payload convergence
    mismatch rejection.
