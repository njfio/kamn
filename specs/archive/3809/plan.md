# Issue #3809 Plan

- Issue: `#3809`
- Status: `Completed`

## Approach
- Expand `MATRIX_ROWS` in the route compatibility contract lane to include additional service API and observability route classes.
- Add deterministic parity/fail-closed checkpoint markers to run-lane and policy validation paths.
- Extend policy regression tamper checks to cover both route mismatch and checkpoint marker drift.
- Update architecture/CI docs with matrix coverage and marker contracts consumed by contract-lane checks.

## Affected Modules
- `scripts/runtime/service_api_observability_route_compatibility_contract.py`
- `scripts/runtime/test_validate_service_api_observability_route_compatibility_live.sh`
- `scripts/runtime/test_check_service_api_observability_route_compatibility_live_policy.sh`
- `scripts/runtime/test_validate_service_api_observability_route_compatibility_live_contract_lane.sh`
- `docs/architecture/service-runtime.md`
- `docs/ci/strategy.md`
- `docs/observability/schema.md`

## Risks and Mitigations
- Risk: Marker/row drift breaks downstream policy parsing.
- Mitigation: Keep reason taxonomy stable and add explicit regression tamper checks in policy tests.
- Risk: Expanded matrix accidentally references non-existent selectors.
- Mitigation: Use existing `kamn-node` selectors already enforced by endpoint tests.

## Interface Contract
- No new external command entrypoints.
- Existing lane/policy scripts remain backward compatible with additional deterministic markers and expanded matrix rows.

## ADR
- No ADR required.
