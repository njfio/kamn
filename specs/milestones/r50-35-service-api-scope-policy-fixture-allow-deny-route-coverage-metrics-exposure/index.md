# R50.35 - Service API Scope-Policy Fixture Allow/Deny Route Coverage Metrics Exposure

## Milestone Summary
Expose Service API scope-policy fixture allow/deny route-coverage markers in runtime `/metrics` by deriving unique route counts for allow and deny fixture rows.

## Contract Markers
- `r50_35_scope_policy_fixture_allow_deny_route_coverage_schema_version=kamn.review.r50.service-api-scope-policy-fixture-allow-deny-route-coverage.v1`
- `r50_35_scope_policy_fixture_allow_deny_route_coverage_issue=#5539`
- `r50_35_scope_policy_fixture_allow_deny_route_coverage_metrics=kamn_service_api_scope_policy_fixture_unique_allow_route_count,kamn_service_api_scope_policy_fixture_unique_deny_route_count`
- `r50_35_scope_policy_fixture_allow_deny_route_coverage_source=fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`

## Scope
- Add RED assertions for fixture unique allow/deny route markers in endpoint `/metrics` tests.
- Derive cardinality values from canonical fixture rows.
- Emit runtime metrics markers for fixture unique allow/deny route coverage counts.

## Exit Criteria
- Issue `#5539` merged.
- `/metrics` exposes both fixture allow/deny route-coverage markers with canonical values.
- Targeted endpoint validation lanes pass.
