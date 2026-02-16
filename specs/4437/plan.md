# Plan: Issue #4437

Status: In Progress
Issue: #4437

## Approach

1. Extend Rust SDK subscription contract test with required taxonomy marker assertions.
2. Extend Python packaging contract/live tests with publish-readiness taxonomy assertions.
3. Add docs marker checks in SDK test scripts.
4. Execute tests to capture deterministic RED failures.

## Affected Modules

- `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
- `scripts/sdk/test_run_python_sdk_packaging_contract.sh`
- `scripts/sdk/test_validate_python_sdk_packaging_live.sh`

## Risks / Mitigations

- Risk: false negatives due marker-name mismatch.
  - Mitigation: pin exact marker strings and versions shared with implementation constants.

## Interfaces / Contracts

- RED-required marker keys:
  - `subscription_reason_taxonomy_version`
  - `subscription_reason_codes_csv`
  - `packaging_publish_readiness_reason_taxonomy_version`
  - `packaging_publish_readiness_reason_codes_csv`

## ADR

No ADR required.
