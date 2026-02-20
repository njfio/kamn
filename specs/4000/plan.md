# Issue #4000 Plan

Status: Reviewed (agent-authored; human review requested in PR)

## Approach
1. Add a matrix fixture JSON with schema marker and entries keyed by `workload` + `lane`.
2. Add workload-aware lookup in `generate_performance_smoke_report.sh`.
3. Add fail-closed validation for required schema/fields.
4. Extend shell tests to add RED checks for workload support and schema drift.
5. Add docs mapping table.

## Affected Paths
- `fixtures/ci/performance_hot_path_fixture_matrix.json`
- `scripts/ci/generate_performance_smoke_report.sh`
- `scripts/ci/test_generate_performance_smoke_report.sh`
- `docs/foundation/observability-slo-dashboards.md`

## Risks and Mitigations
- Risk: shell LOC increase.
  - Mitigation: keep logic localized to existing script/test; no new wrapper executable.
- Risk: fragile parsing.
  - Mitigation: use deterministic Python JSON extraction/validation with explicit error paths.

## Validation
- Run updated script tests.
- Run one docs contract test that asserts alert/SLO policy doc markers remain intact.
