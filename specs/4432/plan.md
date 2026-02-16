# Plan: Issue #4432

Status: In Progress
Issue: #4432

## Approach

1. Add RED assertions for subscription/packaging taxonomy marker surfaces in existing SDK contract
   tests.
2. Implement deterministic subscription taxonomy markers in Rust SDK contract/live scripts.
3. Implement deterministic packaging publish-readiness taxonomy markers in Python SDK
   contract/live scripts.
4. Update SDK docs (`docs/sdk/rust-sdk.md`, `docs/sdk/python-sdk.md`, `docs/sdk/README.md`).
5. Verify with targeted shell/rust/python + lint/format gates; open and merge PR.

## Affected Modules

- `scripts/sdk/run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/validate_rust_sdk_service_client_live.sh`
- `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
- `scripts/sdk/run_python_sdk_packaging_contract.sh`
- `scripts/sdk/test_run_python_sdk_packaging_contract.sh`
- `scripts/sdk/validate_python_sdk_packaging_live.sh`
- `scripts/sdk/test_validate_python_sdk_packaging_live.sh`
- `docs/sdk/rust-sdk.md`
- `docs/sdk/python-sdk.md`
- `docs/sdk/README.md`

## Risks / Mitigations

- Risk: mismatch between contract and live taxonomy marker values.
  - Mitigation: define fixed constants in scripts and assert exact markers in tests.
- Risk: additive marker outputs affect downstream parsing.
  - Mitigation: keep existing markers intact and add deterministic additive fields only.

## Interfaces / Contracts

- Subscription taxonomy:
  - `subscription_reason_taxonomy_version=kamn.sdk.websocket-subscription-reason-taxonomy.v1`
  - `subscription_reason_codes_csv=service_api_websocket_upgrade_required,service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_route_not_found,service_api_method_not_allowed`

- Packaging publish-readiness taxonomy:
  - `packaging_publish_readiness_reason_taxonomy_version=kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1`
  - `packaging_publish_readiness_reason_codes_csv=python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed`

## ADR

No ADR required.
