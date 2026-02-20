# Issue #4057 Plan

- Issue: #4057
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Approach
1. Add a compact request-path fixture matrix in `service_api_endpoint_tests.rs` covering public/protected method+path combinations.
2. Add a unit check on `route_requires_auth` for matrix determinism.
3. Add integration coverage that exercises protected/public routes without auth headers and asserts fail-closed reason taxonomy for protected paths.
4. Add a docs parity block in `docs/ci/strategy.md` and synchronize equivalent markers in `docs/ops/configuration.md`.
5. Extend `crates/kamn-core/tests/ci_strategy_docs.rs` to enforce auth reason taxonomy parity/remediation coverage against `crates/kamn-node/src/service_api_endpoint.rs` constants.
6. Run targeted tests and set `specs/4057/spec.md` status to `Implemented`.

## Affected Files
- `specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md`
- `specs/4057/spec.md`
- `specs/4057/plan.md`
- `specs/4057/tasks.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`

## Risks and Mitigations
- Risk: route matrix drifts from source auth decision logic.
  - Mitigation: unit matrix tests directly call `route_requires_auth`.
- Risk: docs reason taxonomy drifts from source constants.
  - Mitigation: docs tests parse expected taxonomy constants from `service_api_endpoint.rs` and compare exact strings.
- Risk: shell LOC expands via new governance scripts.
  - Mitigation: Rust-test/docs-only implementation; no shell/python/workflow changes.

## Interface Contract
- Authz docs parity markers:
  - `service_api_request_path_authz_reason_taxonomy_version=<version>`
  - `service_api_request_path_authz_reason_codes_csv=<csv>`
  - `service_api_request_path_authz_public_routes_csv=<csv>`
  - `service_api_request_path_authz_protected_routes_csv=<csv>`
  - `service_api_request_path_authz_missing_header_reason_code=<reason>`
  - `service_api_request_path_authz_ops_doc_path=docs/ops/configuration.md`
  - `service_api_request_path_authz_strategy_doc_path=docs/ci/strategy.md`
  - `service_api_request_path_authz_remediation_map_version=v1`
  - `service_api_request_path_authz_remediation.<reason_code>=<action>`

## ADR
- Not required (no dependency/protocol/schema introduction).
