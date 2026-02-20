# Issue #4056 Plan

- Issue: #4056
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Approach
1. Complete service API scope-policy checker enforcement in middleware for protected routes after base auth succeeds.
2. Add deterministic scope fixture matrix file and parser helper coverage in `service_api_endpoint_tests.rs`.
3. Add integration checks for missing/invalid/mismatched/matching scope headers on protected requests.
4. Add scope-policy docs parity/remediation markers in `docs/ci/strategy.md` and `docs/ops/configuration.md`.
5. Extend `crates/kamn-core/tests/ci_strategy_docs.rs` with scope-policy taxonomy/fixture/remediation parity tests.
6. Run targeted verification commands (`kamn-node` scope tests + `ci_strategy_docs` scope tests), then run scoped fmt/clippy/test gate.

## Affected Files
- `specs/4056/spec.md`
- `specs/4056/plan.md`
- `specs/4056/tasks.md`
- `fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`

## Risks and Mitigations
- Risk: scope mapping drift between middleware checker and test helper mapping.
  - Mitigation: fixture functional test compares rows against route scope mapping contract.
- Risk: docs taxonomy/remediation drift from source constants.
  - Mitigation: `ci_strategy_docs` enforces exact marker parity against `service_api_endpoint.rs`.
- Risk: shell LOC ratio regression through new governance scripts.
  - Mitigation: Rust tests + docs markers only; no new shell/python/workflow files.

## Interface Contract
- Scope-policy taxonomy markers:
  - `service_api_scope_policy_reason_taxonomy_version=<version>`
  - `service_api_scope_policy_reason_codes_csv=<csv>`
  - `service_api_scope_policy_fixture_schema_version=<version>`
  - `service_api_scope_policy_fixture_path=fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`
  - `service_api_scope_policy_ops_doc_path=docs/ops/configuration.md`
  - `service_api_scope_policy_strategy_doc_path=docs/ci/strategy.md`
  - `service_api_scope_policy_remediation_map_version=v1`
  - `service_api_scope_policy_remediation.<reason_code>=<action>`

## ADR
- Not required (no dependency/schema/protocol changes).
