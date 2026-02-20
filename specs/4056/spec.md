# Issue #4056 Spec

- Title: Task: implement authorization scope-policy checker with deterministic fail-closed taxonomy
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Problem Statement
Protected service routes require deterministic authorization scope enforcement. Without a fail-closed scope checker, malformed or mismatched scope headers can bypass policy intent and produce non-auditable behavior.

## Scope
In scope:
- Enforce deterministic scope-policy checks for protected service routes.
- Add deterministic fixture matrix and parser contract for allow/deny scope combinations.
- Add reason taxonomy constants and fail-closed rejection reason codes for missing/invalid/mismatched scopes.
- Add docs parity and remediation marker checks for strategy/ops synchronization.

Out of scope:
- External RBAC/directory integration.
- New auth mechanism or token model.
- Shell/python/workflow governance additions.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 230
- shell_to_rust_ratio_delta_estimate: -0.0003
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Protected service routes enforce deterministic required scopes and fail closed on violations.
- AC-2: Scope-policy fixture matrix and parser helper contracts deterministically cover allow/deny combinations.
- AC-3: Scope-policy reason taxonomy and remediation docs markers remain synchronized across source, strategy docs, and ops docs.
- AC-4: Unit, Functional, Integration, and Regression checks for scope-policy behavior and docs parity pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | signed request missing `x-kamn-authz-scope` | `401 unauthorized` + `service_api_auth_scope_header_missing` |
| C-02 | AC-1 | Integration | signed request with invalid/empty scope | `401 unauthorized` + `service_api_auth_scope_invalid` |
| C-03 | AC-1 | Integration | signed request with mismatched route scope | `401 unauthorized` + `service_api_auth_scope_route_mismatch` |
| C-04 | AC-1 | Functional | signed request with matching route scope | protected request accepted (`202`) |
| C-05 | AC-2 | Unit | scope-policy fixture metadata and rows | schema/taxonomy markers match source constants; representative allow/deny rows present |
| C-06 | AC-2 | Functional | fixture rows vs route scope mapping | allow rows match required scopes, deny rows mismatch |
| C-07 | AC-3 | Regression | docs strategy/ops scope-policy parity markers | taxonomy/reason/fixture markers match source constants |
| C-08 | AC-3 | Regression | reason-code remediation map coverage | each scope reason code has strategy+ops remediation marker |
| C-09 | AC-4 | Verification | targeted cargo test commands | all listed commands pass |

## Test Mapping
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::unit_service_api_scope_policy_fixture_parser_contract -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_service_api_scope_policy_docs_parity_markers -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_scope_policy_docs_parity_matches_source_taxonomy -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_scope_policy_remediation_markers_cover_reason_codes -- --exact`

## Success Metrics
- Protected service routes reject missing/invalid/mismatched scope headers deterministically.
- Scope-policy fixture/schema/taxonomy markers fail closed on drift.
- Shell surface remains unchanged (`shell_loc_delta_actual=0` target).
