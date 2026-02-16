# Plan: Issue #4471

Status: In Progress
Issue: #4471

## Approach

1. Add failing tests for partial incident evidence convergence, CI smoke overflow, and missing
   local-heavy opt-in.
2. Ensure failures occur due to missing boundary and convergence policy wiring.
3. Preserve tests as regression guards for #4472 implementation.

## Affected Modules

- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`

## Risks / Mitigations

- Risk: flaky timing checks for boundary behavior.
  - Mitigation: use deterministic static boundary constants and explicit argument values.
