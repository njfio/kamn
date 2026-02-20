# Issue #4058 Spec

- Title: Task: implement local-heavy tenant-isolation matrix lane with deterministic artifacts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Problem Statement
Release governance needs deterministic tenant-isolation evidence showing cross-tenant access attempts are rejected fail closed with stable taxonomy. Without a contract lane + policy checker pair, isolation drift can silently pass.

## Scope
In scope:
- Add tenant-isolation matrix lane report generator (`dry-run` + opt-in `run`).
- Add tenant-isolation policy checker with fail-closed leakage taxonomy.
- Add contract-lane composition (`run-lane` + `check-policy` + tamper rejection + docs parity).
- Add docs parity markers in CI strategy and ops configuration.
- Add Rust contract tests covering lane/policy/contract integration and regression drift checks.

Out of scope:
- Multi-region tenancy drills.
- New shell script bodies or workflow-lane expansion.
- External audit/SIEM integration.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 900
- rust_loc_delta_estimate: 550
- shell_to_rust_ratio_delta_estimate: 0.0019
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Tenant-isolation matrix lane emits deterministic report schema with stable cross-tenant leakage rows.
- AC-2: Policy checker fails closed on leakage marker drift, schema drift, and invalid ci-fast-gate posture.
- AC-3: Contract-lane flow validates docs parity and deterministic tamper rejection reason codes.
- AC-4: Unit, Functional, Integration, Regression, and Performance checks pass while shell-surface guardrails remain `GO` and wrappers stay `exec_dispatch` based.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `run-lane --mode dry-run` | deterministic schema + matrix markers + `execution_reason_code=dry_run_no_commands_executed` |
| C-02 | AC-1 | Functional | `run-lane --mode run` without opt-in | fail-closed rejection with stable opt-in reason |
| C-03 | AC-1 | Functional | `run-lane --mode run` with opt-in | command list executes + deterministic marker/status set |
| C-04 | AC-2 | Unit | valid report to `check-policy` | `final_decision=GO` + policy status `verified` |
| C-05 | AC-2 | Regression | tampered leakage marker row | deterministic NO-GO reason code |
| C-06 | AC-3 | Integration | `run-contract-lane` dry-run composition | GO contract report + policy report + docs parity marker status |
| C-07 | AC-3 | Regression | docs marker omission/parity drift simulation | deterministic fail-closed docs parity reason code |
| C-08 | AC-4 | Integration | wrapper invocations via `exec_dispatch` | wrappers resolve to tenant-isolation contract subcommands |
| C-09 | AC-4 | Performance | contract lane dry-run runtime | bounded under configured budget |

## Test Mapping
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract unit_tenant_isolation_matrix_lane_dry_run_emits_deterministic_schema_and_markers -- --exact`
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract functional_tenant_isolation_matrix_lane_run_mode_requires_explicit_opt_in -- --exact`
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract functional_tenant_isolation_matrix_policy_accepts_deterministic_report -- --exact`
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract regression_tenant_isolation_matrix_policy_rejects_tampered_leakage_marker -- --exact`
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract integration_tenant_isolation_matrix_contract_lane_composes_lane_policy_and_docs_parity -- --exact`
- `cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract performance_tenant_isolation_matrix_contract_lane_dry_run_stays_within_budget -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_runtime_service_api_tenant_isolation_matrix_contract_lane_ci_mode_markers -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_tenant_isolation_matrix_markers -- --exact`

## Success Metrics
- Deterministic tenant-isolation matrix artifacts with stable schema/taxonomy markers.
- Cross-tenant leakage marker drift is fail-closed with explicit reason codes.
- Docs contract parity for strategy + ops remains enforced by Rust tests.
- Shell-surface telemetry remains `GO` (`script_budget_status=pass`) after lane integration.
