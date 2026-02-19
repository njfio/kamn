# Tasks: Issue #4471

Status: Completed
Issue: #4471

## Ordered Tasks

T1 (RED):
- Add convergence-gap and boundary-bypass RED tests in:
  - `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- Capture failing output before implementation.

T2 (GREEN):
- Complete implementation support so new incident go/no-go RED tests pass.

T3 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --incident-readiness-report-file ... --incident-readiness-max-age-seconds 1800`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Failed with:
      - `expected go/no-go evidence contract lane to emit incident boundary reason taxonomy status marker`

- GREEN command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`

- Regression summary:
  - RED coverage now enforces partial incident-readiness evidence fail-closed convergence.
  - RED coverage now enforces CI smoke overflow and missing local-heavy opt-in boundary failure
    semantics for incident go/no-go lanes.
