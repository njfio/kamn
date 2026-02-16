# SDK Contract Overview

This directory captures SDK contract surfaces for Rust/Python/TypeScript clients and deterministic
release-governance markers.

## Core Client Contracts

- Rust HTTP/WebSocket service helper contract:
  - `docs/sdk/rust-sdk.md`
  - `bash scripts/sdk/run_rust_sdk_service_client_contract.sh`
  - `bash scripts/sdk/validate_rust_sdk_service_client_live.sh`
- Python SDK packaging and transport adapter contract:
  - `docs/sdk/python-sdk.md`
  - `bash scripts/sdk/run_python_sdk_packaging_contract.sh`
  - `bash scripts/sdk/validate_python_sdk_packaging_live.sh`
- Cross-language parity contract:
  - `docs/sdk/parity-matrix.md`
  - `bash scripts/sdk/run_cross_language_sdk_parity_matrix.sh`

## Rust HTTP Request-Error Taxonomy Markers

- `request_error_reason_taxonomy_version=kamn.sdk.rust-http-request-error-reason-taxonomy.v1`
- `request_error_reason_codes_csv=service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_route_not_found,service_api_method_not_allowed,service_api_legacy_unauthorized,service_api_legacy_conflict,service_api_legacy_bad_request,service_api_legacy_error_unknown`
- `request_error_taxonomy_status=verified`
- `http_error_taxonomy_contract_status=verified`

## Rust WebSocket Subscription Taxonomy Markers

- `subscription_reason_taxonomy_version=kamn.sdk.websocket-subscription-reason-taxonomy.v1`
- `subscription_reason_codes_csv=service_api_websocket_upgrade_required,service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_route_not_found,service_api_method_not_allowed`
- `subscription_taxonomy_status=verified`
- `subscription_contract_status=verified`

## Python Packaging Publish-Readiness Taxonomy Markers

- `packaging_publish_readiness_reason_taxonomy_version=kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1`
- `packaging_publish_readiness_reason_codes_csv=python_packaging_metadata_missing,python_packaging_metadata_invalid,python_packaging_import_probe_failed,python_packaging_unittest_contract_failed`
- `packaging_publish_readiness_status=verified`
- `publish_readiness_taxonomy_status=verified`
