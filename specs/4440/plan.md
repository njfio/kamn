# Plan: Issue #4440

Status: Completed
Issue: #4440

## Approach

1. Introduce fixed packaging taxonomy constants and reason-code CSV in compose-topology lane output.
2. Extend policy checker to validate packaging taxonomy/evidence markers and emit deterministic
   mismatch reasons.
3. Update deploy/CI docs to document marker surface and fail-closed behavior.
4. Verify with existing compose-topology lane/policy tests.

## Affected Modules

- `scripts/deploy/validate_compose_topology_contract_lane.sh`
- `scripts/deploy/check_compose_topology_contract_policy.sh`
- `docs/ops/deployment.md`
- `docs/ci/strategy.md`

## Risks / Mitigations

- Risk: new taxonomy fields diverge between lane and policy checker.
  - Mitigation: define constants in scripts and test exact expected values in both outputs.
- Risk: reason-code list growth destabilizes downstream parsing.
  - Mitigation: maintain ordered deterministic CSV and explicit docs markers.

## Interfaces / Contracts

- Packaging taxonomy version:
  - `kamn.deploy.compose-packaging-reason-taxonomy.v1`
- Deterministic reason CSV:
  - `compose_packaging_manifest_drift_detected,compose_packaging_config_drift_detected,compose_packaging_evidence_contract_drift_detected`

## ADR

No ADR required.
