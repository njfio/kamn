# Issue #4058 Plan

- Issue: #4058
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Approach
1. Implement `scripts/runtime/service_api_tenant_isolation_matrix_live_contract.py` with three subcommands:
   - `run-lane` (dry-run/run, local-heavy opt-in guard, deterministic matrix schema/report output).
   - `check-policy` (fail-closed taxonomy checks for leakage/schema/marker drift).
   - `run-contract-lane` (lane + policy composition, tamper simulation, docs parity checks, bounded runtime).
2. Add `exec_dispatch` wrappers and `scripts/lib/exec_registry.json` entries for:
   - `validate_service_api_tenant_isolation_matrix_live.sh`
   - `check_service_api_tenant_isolation_matrix_live_policy.sh`
   - `validate_service_api_tenant_isolation_matrix_live_contract_lane.sh`
3. Add Rust contract tests in `crates/kamn-core/tests/service_api_tenant_isolation_matrix_contract.rs` for unit/functional/integration/regression/performance coverage.
4. Add docs parity markers/commands:
   - `docs/ci/strategy.md`
   - `docs/ops/configuration.md`
   - plus Rust docs-contract assertions in `ci_strategy_docs.rs` and `service_api_ops_configuration_docs.rs`.
5. Run targeted validation (fmt, clippy scoped to touched crates/tests, targeted cargo tests).

## Affected Files
- `specs/4058/spec.md`
- `specs/4058/plan.md`
- `specs/4058/tasks.md`
- `scripts/runtime/service_api_tenant_isolation_matrix_live_contract.py`
- `scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh`
- `scripts/runtime/check_service_api_tenant_isolation_matrix_live_policy.sh`
- `scripts/runtime/validate_service_api_tenant_isolation_matrix_live_contract_lane.sh`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/service_api_tenant_isolation_matrix_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md`

## Risks and Mitigations
- Risk: introducing new shell bodies worsens shell-ratio headroom.
  - Mitigation: wrappers are `exec_dispatch` symlinks (0 shell LOC), behavior in Python + Rust tests.
- Risk: drift between docs markers and checker taxonomy.
  - Mitigation: Rust docs-contract tests assert exact markers/commands.
- Risk: run-mode lane is accidentally allowed without local opt-in.
  - Mitigation: explicit opt-in env guard + functional test for fail-closed behavior.

## Interface Contract
- Run report schema: `kamn.runtime.service-api-tenant-isolation-matrix-live-report.v1`
- Policy report schema: `kamn.runtime.service-api-tenant-isolation-matrix-live-policy-report.v1`
- Contract report schema: `kamn.runtime.service-api-tenant-isolation-matrix-live-contract-lane-report.v1`
- Matrix schema: `kamn.runtime.service-api-tenant-isolation-matrix.v1`
- Reason taxonomy marker:
  - `service_api_tenant_isolation_matrix_reason_taxonomy_version=<version>`
- Reason code surface:
  - `service_api_tenant_isolation_matrix_reason_codes_csv=<csv>`

## ADR
- Not required (no dependency/schema/protocol expansion beyond local lane artifacts).
