# R50.31 - Service API Scope-Policy Fixture Metadata Parity Metrics Exposure

## Milestone Summary
Expose Service API scope-policy fixture metadata parity markers in runtime `/metrics` by deriving fixture reason taxonomy version and reason-code cardinality from canonical fixture metadata.

## Contract Markers
- `r50_31_scope_policy_fixture_metadata_parity_schema_version=kamn.review.r50.service-api-scope-policy-fixture-metadata-parity.v1`
- `r50_31_scope_policy_fixture_metadata_parity_issue=#5531`
- `r50_31_scope_policy_fixture_metadata_parity_metrics=kamn_service_api_scope_policy_fixture_reason_taxonomy_info,kamn_service_api_scope_policy_fixture_reason_code_count`
- `r50_31_scope_policy_fixture_metadata_parity_source=fixtures/runtime/service_api_scope_policy_fixture_matrix.txt`

## Scope
- Add RED assertions for fixture metadata parity markers in endpoint `/metrics` tests.
- Derive parity values from canonical fixture metadata.
- Emit runtime metrics markers for fixture reason taxonomy version and reason-code count.

## Exit Criteria
- Issue `#5531` merged.
- `/metrics` exposes both fixture metadata parity markers with canonical values.
- Targeted endpoint validation lanes pass.
