# Plan: #4367 Implementation

## Approach

- Extend milestone bundle validation logic in `scripts/deploy/gonogo_evidence_contract.py` to validate:
  - `rotation_preflight_reason_taxonomy_version`
  - `rotation_preflight_reason_codes_csv`
  - `rotation_preflight_reason_codes_value`
  - `lane_mode`, `ci_fast_gate_eligible`, `ci_fast_gate_scope`, `fast_gate_exclusion_status`, `fast_gate_exclusion_reason_code`, `run_mode_command_status`
- Add deterministic reason codes and expected contract markers in observed/contracts sections.
- Keep JSON schema unchanged while tightening required deterministic semantics.

## Risks

- Existing fixtures may miss new fields.

## Mitigation

- Update deploy test fixtures and integration fixture generation flows.
