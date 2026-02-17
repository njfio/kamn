# Plan — #4412

Status: Reviewed

## Approach

- Introduce deterministic telemetry evidence-link reason codes in policy taxonomy constants.
- Enforce run-mode evidence-link completeness (required keys and valid files) before GO.
- Validate linked artifact schema/status convergence against expected telemetry contracts.
- Keep normalized reason output (`reason_codes_value`) deterministic across pass/fail paths.
- Update telemetry contract-lane tests and CI strategy docs to reflect the expanded governance surface.

## Affected Areas

- `scripts/runtime/unified_api_observability_local_heavy_live_contract.py`
- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations

- Risk: run-mode evidence links point to ephemeral temp files and fail new checks.
  - Mitigation: persist/copied artifact files into stable paths referenced by the summary report.
- Risk: broad CSV taxonomy strings drift across tests/docs.
  - Mitigation: update all expected marker assertions in the same change.

