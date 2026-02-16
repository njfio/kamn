# Plan: Issue #4436

Status: Completed
Issue: #4436

## Approach

1. Define fixed taxonomy constants in Rust SDK contract/live scripts.
2. Emit markers in stdout and JSON report payloads for both scripts.
3. Extend Python regression suite for deterministic legacy reason normalization behavior.
4. Update SDK docs to include taxonomy/evidence marker references.
5. Verify with targeted shell/rust/python and formatter/lint gates.

## Affected Modules

- `scripts/sdk/run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/validate_rust_sdk_service_client_live.sh`
- `tests/python/test_sdk.py`
- `docs/sdk/rust-sdk.md`
- `docs/sdk/README.md`

## Risks / Mitigations

- Risk: mismatch between runner and validation taxonomy values.
  - Mitigation: define constants and verify exact values in both test scripts.
- Risk: Python fallback normalization behavior unintentionally changes existing callers.
  - Mitigation: preserve current behavior for existing reasons and add regression coverage.

## Interfaces / Contracts

- Taxonomy version:
  - `kamn.sdk.rust-http-request-error-reason-taxonomy.v1`
- Reason CSV:
  - `service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_route_not_found,service_api_method_not_allowed,service_api_legacy_unauthorized,service_api_legacy_conflict,service_api_legacy_bad_request,service_api_legacy_error_unknown`

## ADR

No ADR required.
