# Plan — #4297

Status: Reviewed

## Approach

- Extend `scripts/runtime/unified_api_observability_local_heavy_live_contract.py` to:
  - emit deterministic correlation schema markers in run-lane report payload and stdout markers.
  - enforce required correlation schema fields and API/runtime/Kolme propagation parity in policy checks.
  - project deterministic reason codes for schema/version drift and propagation mismatch classes.
- Add RED tests first in existing unified local-heavy test harnesses:
  - `scripts/runtime/test_validate_unified_api_observability_local_heavy_live.sh`
  - `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
  - `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
- Update docs and docs-contract assertions for new correlation markers/reasons:
  - `docs/observability/schema.md`
  - `docs/foundation/release-gonogo-checklist.md`
  - `crates/kamn-core/tests/observability_schema_docs.rs`

## Affected Areas

- `scripts/runtime/unified_api_observability_local_heavy_live_contract.py`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live.sh`
- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
- `docs/observability/schema.md`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/observability_schema_docs.rs`

## Risks and Mitigations

- Risk: reason taxonomy expansion causes docs/test drift.
  - Mitigation: centralize constants in contract script and update docs/tests in same change.
- Risk: run-mode artifact policy checks become brittle.
  - Mitigation: keep existing evidence checks intact; layer correlation checks on unified report markers.
- Risk: CI runtime budget regression.
  - Mitigation: avoid introducing additional nested commands or new lane executables.

## Interfaces and Contracts

- Unified run-lane correlation schema contract:
  - deterministic schema version marker
  - deterministic required-field CSV marker
  - deterministic API/runtime/Kolme correlation-id markers
- Policy fail-closed reason classes:
  - required-field mismatch
  - schema-version mismatch
  - correlation propagation mismatch
