# R50.32 - Service API Scope-Policy Fixture Coverage Cardinality Metrics Exposure

## Milestone Summary
Expose Service API scope-policy fixture coverage cardinality markers in runtime `/metrics` by deriving unique route and unique scope counts from canonical fixture rows.

## Contract Markers
- `r50_32_scope_policy_fixture_coverage_schema_version=kamn.review.r50.service-api-scope-policy-fixture-coverage-cardinality.v1`
- `r50_32_scope_policy_fixture_coverage_issue=#5533`
- `r50_32_scope_policy_fixture_coverage_metrics=kamn_service_api_scope_policy_fixture_unique_route_count,kamn_service_api_scope_policy_fixture_unique_scope_count`
- `r50_32_scope_policy_fixture_coverage_source=fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`

## Scope
- Add RED assertions for fixture unique route/scope count markers in endpoint `/metrics` tests.
- Derive cardinality values from canonical fixture rows.
- Emit runtime metrics markers for fixture unique route/scope coverage counts.

## Exit Criteria
- Issue `#5533` merged.
- `/metrics` exposes both fixture coverage cardinality markers with canonical values.
- Targeted endpoint validation lanes pass.
