# Plan: Issue #4438

Status: In Progress
Issue: #4438

## Approach

1. Add deterministic taxonomy constants to Rust subscription and Python packaging scripts.
2. Emit taxonomy/evidence markers in stdout and JSON reports for contract/live scripts.
3. Update SDK docs to include subscription/packaging taxonomy marker references.
4. Verify against RED assertions and full targeted suites.

## Affected Modules

- `scripts/sdk/run_rust_sdk_service_client_contract.sh`
- `scripts/sdk/validate_rust_sdk_service_client_live.sh`
- `scripts/sdk/run_python_sdk_packaging_contract.sh`
- `scripts/sdk/validate_python_sdk_packaging_live.sh`
- `docs/sdk/rust-sdk.md`
- `docs/sdk/python-sdk.md`
- `docs/sdk/README.md`

## Risks / Mitigations

- Risk: taxonomy constants diverge between scripts.
  - Mitigation: define fixed constants in each script and assert exact values in tests.
- Risk: evidence marker additions break downstream parsers.
  - Mitigation: maintain existing markers and add fields additively.

## Interfaces / Contracts

- Subscription taxonomy version:
  - `kamn.sdk.websocket-subscription-reason-taxonomy.v1`
- Packaging publish-readiness taxonomy version:
  - `kamn.sdk.python-packaging-publish-readiness-reason-taxonomy.v1`

## ADR

No ADR required.
