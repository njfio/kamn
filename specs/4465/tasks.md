# Tasks: Issue #4465

Status: Completed
Issue: #4465

## Ordered Tasks

T1 (RED, Functional/Regression):
- Add audit-integrity red tests in `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- Capture failing markers before implementation.

T2 (GREEN):
- Complete implementation support so new red tests pass.

T3 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --audit-integrity-report-file ... --audit-integrity-max-age-seconds 1800`

- GREEN command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`

- Regression summary:
  - Red tests now cover audit-integrity unstable-source taxonomy drift and tampered gate payload
    convergence mismatch.
  - Fast-lane deploy contract tests exercise audit-integrity gate arguments and GO decision markers.
