# Tasks: Issue #4467

Status: Completed
Issue: #4467

## Ordered Tasks

T1 (RED):
- Add SLO threshold drift + gate-mismatch tests in
  `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- Capture failing output before implementation.

T2 (GREEN):
- Complete implementation support so new SLO red tests pass.

T3 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --slo-policy-report-file ... --slo-policy-max-age-seconds 1800`

- GREEN command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`

- Regression summary:
  - Red tests now cover SLO threshold drift (`gonogo_slo_policy_reason_key_mismatch`) and tampered
    SLO policy gate payload convergence mismatch rejection.
