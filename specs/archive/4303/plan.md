# Plan — #4303

Status: Reviewed

## Approach

- Implement correlation schema constants and deterministic reason-code extensions in
  `scripts/runtime/unified_api_observability_local_heavy_live_contract.py`.
- Extend run-lane payload/stdout with correlation contract markers.
- Extend policy checker with:
  - required-marker validation
  - schema-version validation
  - API/runtime/Kolme propagation parity validation
- Keep deterministic reason taxonomy ordering synchronized across:
  - contract constants
  - shell test expected CSV strings
  - docs marker references

## Affected Areas

- `scripts/runtime/unified_api_observability_local_heavy_live_contract.py`
- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
- `docs/observability/schema.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/observability_schema_docs.rs`

## Risks and Mitigations

- Risk: deterministic reason ordering drift.
  - Mitigation: reason codes remain sourced from a single ordered constant.
- Risk: checker surface grows without reuse.
  - Mitigation: keep helper functions local and narrow to correlation schema concerns only.
