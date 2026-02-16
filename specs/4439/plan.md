# Plan: Issue #4439

Status: In Progress
Issue: #4439

## Approach

1. Add RED checks to existing compose-topology lane tests for required packaging taxonomy markers.
2. Add tamper-path checks for policy report taxonomy/reason CSV mismatch.
3. Capture deterministic RED failures before implementation wiring.

## Affected Modules

- `scripts/deploy/test_validate_compose_topology_contract_lane.sh`
- `scripts/deploy/test_check_compose_topology_contract_policy.sh`

## Risks / Mitigations

- Risk: Tests become brittle from unordered reason CSV comparisons.
  - Mitigation: compare exact expected deterministic CSV outputs with fixed ordering.
- Risk: Test expansion increases runtime significantly.
  - Mitigation: reuse existing fixtures and tamper temporary JSON only.

## Interfaces / Contracts

- Required markers under test:
  - `packaging_reason_taxonomy_version`
  - `packaging_reason_codes_csv`
  - `packaging_contract_evidence_status`

## ADR

No ADR required.
