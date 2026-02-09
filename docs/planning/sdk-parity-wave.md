# SDK Parity and Workflow Automation Wave (Issue #926 / #940)

This plan tracks deterministic SDK workflow automation contracts that keep local and CI validation fast, reproducible, and low cost.

## Scope
- Shared SDK parity fixture matrix remains the source of truth:
  - `fixtures/sdk_parity/register_validation_cases.json`
- Generated fixture snapshot contract:
  - `fixtures/sdk_parity/register_validation_snapshot.json`
- Drift checker command:
  - `python3 scripts/sdk/check_example_fixture_drift.py --fixture fixtures/sdk_parity/register_validation_cases.json --snapshot fixtures/sdk_parity/register_validation_snapshot.json --output-json /tmp/sdk-example-fixture-drift-report.json`
- Drift policy checker command:
  - `bash scripts/sdk/check_example_fixture_drift_policy.sh --report-file /tmp/sdk-example-fixture-drift-report.json`
- Contract lane command:
  - `bash scripts/sdk/run_example_fixture_drift_contract_lane.sh --output-report /tmp/sdk-example-fixture-drift-contract-report.json`

## SDK Example Fixture Drift Checker Contract (Issue #940)
- Checker report schema:
  - `kamn.sdk.example-fixture-drift-report.v1`
- Snapshot schema:
  - `kamn.sdk.example-fixture-snapshot.v1`
- Contract lane reason key:
  - `sdk_example_fixture_drift_reason_codes:GO:v1`
- Runtime budget environment variable:
  - `KAMN_SDK_EXAMPLE_FIXTURE_DRIFT_MAX_SECONDS=45` (default)
- Cost policy:
  - lane executes only shared fixture + bounded matrix checks.
  - no deep/scheduled lanes in PR critical path.

Fail-closed regression marker:
- fixture snapshot drift, schema mismatch, or missing planning/doc parity references force `NO-GO` (`Regression: #940`).

## CI Routing Contract
- Selector paths that must route to SDK bounded scope:
  - `scripts/sdk/check_example_fixture_drift.py`
  - `scripts/sdk/check_example_fixture_drift_policy.sh`
  - `scripts/sdk/run_example_fixture_drift_contract_lane.sh`
  - `scripts/sdk/test_check_example_fixture_drift.sh`
  - `scripts/sdk/test_check_example_fixture_drift_policy.sh`
  - `scripts/sdk/test_run_example_fixture_drift_contract_lane.sh`
  - `docs/planning/sdk-parity-wave.md`
  - `fixtures/sdk_parity/register_validation_snapshot.json`

## Local Validation
Run from repository root:

```bash
python3 scripts/sdk/check_example_fixture_drift.py --fixture fixtures/sdk_parity/register_validation_cases.json --snapshot fixtures/sdk_parity/register_validation_snapshot.json --output-json /tmp/sdk-example-fixture-drift-report.json
bash scripts/sdk/check_example_fixture_drift_policy.sh --report-file /tmp/sdk-example-fixture-drift-report.json
bash scripts/sdk/run_example_fixture_drift_contract_lane.sh --output-report /tmp/sdk-example-fixture-drift-contract-report.json
bash scripts/sdk/test_check_example_fixture_drift.sh
bash scripts/sdk/test_check_example_fixture_drift_policy.sh
bash scripts/sdk/test_run_example_fixture_drift_contract_lane.sh
```
