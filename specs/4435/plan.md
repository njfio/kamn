# Plan: Issue #4435

Status: In Progress
Issue: #4435

## Approach

1. Extend `test_run_rust_sdk_service_client_contract.sh` with taxonomy/evidence marker checks.
2. Extend `test_validate_rust_sdk_service_client_live.sh` with taxonomy/evidence marker checks.
3. Add docs marker checks for SDK HTTP helper taxonomy markers.
4. Run both scripts to capture deterministic RED failures prior to implementation.

## Affected Modules

- `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`

## Risks / Mitigations

- Risk: tests become too coupled to wording.
  - Mitigation: verify fixed marker keys and fixed version/csv values only.

## Interfaces / Contracts

- RED-required marker keys:
  - `request_error_reason_taxonomy_version`
  - `request_error_reason_codes_csv`
  - `request_error_taxonomy_status`

## ADR

No ADR required.
