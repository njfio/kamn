# Plan: Issue #4467

Status: Completed
Issue: #4467

## Approach

1. Add SLO source-report fixtures to go/no-go test suite.
2. Add RED assertions for deterministic SLO taxonomy markers and threshold drift failure codes.
3. Add RED tamper test for checker-side SLO gate convergence mismatch.
4. Keep tests scoped to deploy/go-no-go fast lanes for iteration speed.

## Affected Modules

- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `specs/4467/*`

## Risks and Mitigations

- Risk: test scenarios drift from actual contract source fields.
  - Mitigation: fixtures mirror `kamn.deploy.slo-rollback-report.v1` fields already produced by
    deployment SLO lane contracts.
