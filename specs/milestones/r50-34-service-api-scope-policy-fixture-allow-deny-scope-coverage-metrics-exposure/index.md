# R50.34 - Service API Scope-Policy Fixture Allow/Deny Scope Coverage Metrics Exposure

## Milestone Summary
Expose Service API scope-policy fixture allow/deny scope-coverage markers in runtime `/metrics` by deriving unique scope counts for allow and deny fixture rows.

## Contract Markers
- `r50_34_scope_policy_fixture_allow_deny_scope_coverage_schema_version=kamn.review.r50.service-api-scope-policy-fixture-allow-deny-scope-coverage.v1`
- `r50_34_scope_policy_fixture_allow_deny_scope_coverage_issue=#5537`
- `r50_34_scope_policy_fixture_allow_deny_scope_coverage_metrics=kamn_service_api_scope_policy_fixture_unique_allow_scope_count,kamn_service_api_scope_policy_fixture_unique_deny_scope_count`
- `r50_34_scope_policy_fixture_allow_deny_scope_coverage_source=fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`

## Scope
- Add RED assertions for fixture unique allow/deny scope markers in endpoint `/metrics` tests.
- Derive cardinality values from canonical fixture rows.
- Emit runtime metrics markers for fixture unique allow/deny scope coverage counts.

## Exit Criteria
- Issue `#5537` merged.
- `/metrics` exposes both fixture allow/deny scope-coverage markers with canonical values.
- Targeted endpoint validation lanes pass.
