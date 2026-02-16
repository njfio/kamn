# Plan: Issue #4465

Status: Completed
Issue: #4465

## Approach

1. Extend go/no-go bundle tests with audit-integrity source report fixtures.
2. Add RED assertions for:
   - deterministic audit marker outputs
   - unstable source report rejection
   - tampered bundle audit gate rejection.
3. Keep tests scoped to deploy contract lane surfaces for fast iteration.

## Affected Modules

- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `specs/4465/*`

## Risks and Mitigations

- Risk: tests become aspirational and fail on main.
  - Mitigation: land with implementation in same branch before PR/merge.

## Interfaces / Contracts

- Expected checker mismatch string:
  - `audit integrity gate convergence mismatch`
- Expected deterministic markers:
  - `audit_integrity_reason_taxonomy_version`
  - `audit_integrity_reason_codes_csv`
  - `audit_integrity_reason_codes_value`
