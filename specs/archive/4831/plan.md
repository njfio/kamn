# Plan — Issue #4831

## Approach

1. Add a fail-closed telemetry collector wrapper over `generate_combined_shell_surface_trend_report.sh`.
2. Normalize decision output into deterministic governance markers:
   - `status=ok|fail`
   - `final_decision=GO|NO-GO`
   - taxonomy/version and reason code markers
3. Add a contract test that validates:
   - passing telemetry path
   - deterministic failing path when generator output report is missing
4. Wire the new telemetry collector test into `scripts/ci/test_ci_tools.sh` to keep drift gates active.
5. Update CI strategy docs with collector command and reason-marker contract.

## Affected Modules

- `scripts/ci/collect_shell_rust_loc_telemetry.sh`
- `scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`

## Risks / Mitigations

- Risk: duplicate or unstable reason surfaces across telemetry and policy scripts.
  Mitigation: explicit collector taxonomy version and deterministic reason CSV/value markers.
- Risk: false positives from malformed generator reports.
  Mitigation: fail-closed typed reason mapping and contract test for missing report path.
- Risk: CI runtime growth.
  Mitigation: add one lightweight shell test to existing CI tools suite.

## Interfaces / Contracts

- Collector schema version:
  - `kamn.ci.shell-rust-loc-telemetry-report.v1`
- Collector taxonomy version:
  - `kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1`
- Deterministic marker contract:
  - `status=ok|fail`
  - `final_decision=GO|NO-GO`
  - `reason_codes_csv=<deterministic ordered csv>`
  - `reason_codes=none|<csv>`
  - `reason_codes_value=none|<csv>`
- Failure reason mapping includes:
  - `shell_rust_loc_telemetry_generator_failed`
  - `shell_rust_loc_telemetry_report_missing`
  - `shell_rust_loc_telemetry_report_parse_failed`
  - `shell_rust_loc_telemetry_report_schema_mismatch`
  - `shell_rust_loc_telemetry_metrics_missing`
  - `shell_rust_loc_telemetry_metric_type_invalid`
  - `shell_rust_loc_telemetry_script_budget_status_fail`
  - `shell_rust_loc_telemetry_script_budget_exit_nonzero`

## ADR

No ADR required. No dependency/protocol boundary changes were introduced.
