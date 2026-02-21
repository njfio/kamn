# R50.33 - Service API Scope-Policy Fixture Shape-Cardinality Metrics Exposure

## Milestone Summary
Expose Service API scope-policy fixture shape-cardinality markers in runtime `/metrics` by deriving unique method and unique expected-outcome counts from canonical fixture rows.

## Contract Markers
- `r50_33_scope_policy_fixture_shape_schema_version=kamn.review.r50.service-api-scope-policy-fixture-shape-cardinality.v1`
- `r50_33_scope_policy_fixture_shape_issue=#5535`
- `r50_33_scope_policy_fixture_shape_metrics=kamn_service_api_scope_policy_fixture_unique_method_count,kamn_service_api_scope_policy_fixture_unique_expected_outcome_count`
- `r50_33_scope_policy_fixture_shape_source=fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`

## Scope
- Add RED assertions for fixture unique method/expected-outcome markers in endpoint `/metrics` tests.
- Derive cardinality values from canonical fixture rows.
- Emit runtime metrics markers for fixture unique method/expected-outcome counts.

## Exit Criteria
- Issue `#5535` merged.
- `/metrics` exposes both fixture shape-cardinality markers with canonical values.
- Targeted endpoint validation lanes pass.
