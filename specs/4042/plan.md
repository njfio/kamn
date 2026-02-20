# Issue #4042 Plan

- Issue: #4042
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Approach
1. Add `scripts/runtime/request_response_schema_compatibility_live_contract.py` with:
   - `run-lane`: deterministic request/response schema pair evaluation for compatible/incompatible supported version pairs.
   - `check-policy`: fail-closed report contract validation.
   - `run-contract-lane`: lane + checker + docs parity + tamper rejection.
2. Add fixture metadata file under `fixtures/runtime/` for supported-pair schema-compatibility matrix inputs.
3. Wire `exec_dispatch` wrappers and `scripts/lib/exec_registry.json` entries:
   - `validate_request_response_schema_compatibility_live.sh`
   - `check_request_response_schema_compatibility_live_policy.sh`
   - `validate_request_response_schema_compatibility_live_contract_lane.sh`
4. Add Rust contract tests in `crates/kamn-core/tests/request_response_schema_compatibility_contract.rs` for AC/C-case coverage.
5. Update docs and docs-contract tests:
   - `docs/ci/strategy.md`
   - `docs/ops/configuration.md`
   - `crates/kamn-core/tests/ci_strategy_docs.rs`
   - `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
6. Run targeted `fmt`, `clippy`, and cargo tests.

## Affected Files
- `specs/4042/spec.md`
- `specs/4042/plan.md`
- `specs/4042/tasks.md`
- `fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt`
- `scripts/runtime/request_response_schema_compatibility_live_contract.py`
- `scripts/runtime/validate_request_response_schema_compatibility_live.sh`
- `scripts/runtime/check_request_response_schema_compatibility_live_policy.sh`
- `scripts/runtime/validate_request_response_schema_compatibility_live_contract_lane.sh`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/request_response_schema_compatibility_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`

## Risks and Mitigations
- Risk: reason-code drift across lane, checker, fixtures, and docs.
  - Mitigation: docs parity checks in `run-contract-lane` + Rust docs-contract assertions.
- Risk: shell-surface growth pressure.
  - Mitigation: wrappers remain `exec_dispatch` symlinks; logic implemented in Python + Rust tests.
- Risk: flaky runtime-budget assertions.
  - Mitigation: dry-run-only bounded performance checks with conservative threshold.

## Interface Contract
- Lane schema: `kamn.runtime.request-response-schema-compatibility-report.v1`
- Policy schema: `kamn.runtime.request-response-schema-compatibility-policy-report.v1`
- Contract-lane schema: `kamn.runtime.request-response-schema-compatibility-contract-lane-report.v1`
- Fixture schema: `kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1`
- Reason taxonomy marker:
  - `request_response_schema_compatibility_reason_taxonomy_version=kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1`

## ADR
- Not required (no new dependency or wire-format change; deterministic governance artifact extension only).
