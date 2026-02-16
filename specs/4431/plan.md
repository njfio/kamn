# Plan: Issue #4431

Status: In Progress
Issue: #4431

## Approach

1. Add RED assertions to existing Rust SDK HTTP contract tests for required taxonomy/evidence
   markers and docs marker surfaces.
2. Implement deterministic request-error taxonomy outputs in:
   - `scripts/sdk/run_rust_sdk_service_client_contract.sh`
   - `scripts/sdk/validate_rust_sdk_service_client_live.sh`
3. Add Python regression coverage for deterministic legacy backend reason normalization behavior.
4. Add/update SDK docs marker surfaces (`docs/sdk/rust-sdk.md`, `docs/sdk/README.md`).
5. Verify with targeted shell + rust + python tests, then open/merge PR.

## Affected Modules

- `scripts/sdk/run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/validate_rust_sdk_service_client_live.sh`
- `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
- `tests/python/test_sdk.py`
- `docs/sdk/rust-sdk.md`
- `docs/sdk/README.md`
- `specs/4431/*`
- `specs/4435/*`
- `specs/4436/*`

## Risks / Mitigations

- Risk: Shell tests become brittle due exact marker matching.
  - Mitigation: use fixed taxonomy constants and stable marker names in scripts/docs.
- Risk: Python normalization change breaks existing adapter-error expectations.
  - Mitigation: keep current public reason values where expected; add deterministic fallback tests.
- Risk: Runtime budget regressions in SDK lane scripts.
  - Mitigation: preserve existing max-seconds bounds and only add marker outputs/validation.

## Interfaces / Contracts

- Rust SDK HTTP taxonomy markers:
  - `request_error_reason_taxonomy_version=kamn.sdk.rust-http-request-error-reason-taxonomy.v1`
  - `request_error_reason_codes_csv=service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_route_not_found,service_api_method_not_allowed,service_api_legacy_unauthorized,service_api_legacy_conflict,service_api_legacy_bad_request,service_api_legacy_error_unknown`
  - `request_error_taxonomy_status=verified`

- Live validation governance markers:
  - `http_error_taxonomy_contract_status=verified`
  - `request_error_reason_taxonomy_version=<same-version>`
  - `request_error_reason_codes_csv=<same-csv>`

## ADR

No ADR required (no dependency or wire/protocol migration).
