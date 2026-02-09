# Observability Stack and SLO Dashboard Baseline (Issues #206, #593)

This document captures the first implementation slice for deterministic observability and SLO health reporting.

## Scope Delivered
- Added `crates/kamn-core/src/observability.rs` with:
  - `ObservabilitySample` input model.
  - `ObservabilitySloProfile` baseline thresholds for latency, throughput, error rate, and availability.
  - `ObservabilityMonitor` evaluator with alert generation and historical rollup.
  - `ObservabilityReport` and `ObservabilitySnapshot` outputs.
  - `ObservabilityAlert`, `ObservabilityMetric`, `ObservabilitySeverity`, and `ObservabilityHealth`.
  - `ObservabilityError` typed validation failures.
- Added integration tests in `crates/kamn-core/tests/observability_stack.rs`.
- Added frontend dashboard projection linkage in `packages/kamn-dashboard`:
  - deterministic severity mapping from SLO values to UI badge classes.
  - stale snapshot banner behavior for operator triage.

## SLO Evaluation Rules
- `LatencyP50`: warning when above max threshold.
- `LatencyP99`: critical when above max threshold.
- `Throughput`: warning when below minimum threshold.
- `ErrorRate`:
  - warning when above max threshold.
  - critical when above 2x max threshold.
- `Availability`: critical when below minimum threshold.

## Dashboard Rollup Semantics
- Snapshot fields are deterministic:
  - total sample count.
  - healthy/degraded/critical sample counts.
  - latest health status.
- Overall health for each sample:
  - `Critical` if any critical alert exists.
  - `Degraded` if warning alerts exist without critical alerts.
  - `Healthy` when no alerts exist.
- Frontend dashboard mapping:
  - critical samples map to `severity-critical` badges.
  - stale snapshots map to `stale-data-banner` indicators.
  - unavailable sample fetch maps to `dashboard-error` state.
  - empty sample batches map to `dashboard-empty` state.

## Post-Cutover SLO Gate Evidence Contract (Issue #711)
Launch expansion decisions require deterministic SLO evidence export and fail-closed policy checks.

- Stable shell wrappers:
  - `scripts/canary/generate_post_cutover_slo_evidence_bundle.sh`
  - `scripts/canary/check_post_cutover_slo_policy.sh`
- Shared Python implementation:
  - `scripts/canary/post_cutover_slo_contract.py`
- Evidence bundle generator:
  - `bash scripts/canary/generate_post_cutover_slo_evidence_bundle.sh --output-file /tmp/post-cutover-slo.json --window-minutes 15 --p95-latency-ms 140 --max-p95-latency-ms 200 --error-rate-bps 18 --max-error-rate-bps 25 --delivery-success-bps 9992 --min-delivery-success-bps 9950 --snapshot-age-seconds 30 --max-snapshot-age-seconds 120 --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/canary/check_post_cutover_slo_policy.sh --bundle-file /tmp/post-cutover-slo.json`
- Fast contract lane:
  - `bash scripts/canary/run_post_cutover_slo_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/canary/run_post_cutover_slo_deep_lane.sh --output-json post-cutover-slo-report.json`
- Regression policy:
  - stale snapshots and incomplete SLO evidence force `NO-GO` (`Regression: #711`).

## SLO/Alert Evidence Policy Checker Contract
Operational launch readiness now enforces deterministic alert-schema evidence and fail-closed drift detection for SLO bundles.

- Contract lane command:
  - `bash scripts/canary/run_post_cutover_slo_contract_lane.sh`
- Deep lane command:
  - `bash scripts/canary/run_post_cutover_slo_deep_lane.sh --output-json post-cutover-slo-report.json`

Runtime budget controls:

- `KAMN_POST_CUTOVER_SLO_MAX_SECONDS`
- `KAMN_POST_CUTOVER_SLO_DEEP_MAX_SECONDS`

Required reason-key markers:

- `slo_alert_reason_codes:GO:v1`
- `slo_alert_reason_codes:NO-GO:v1`

Regression policy:

- missing or drifted alert evidence schema/keys must fail closed (`Regression: #913`).

## Dashboard Stale/Error Budget Policy Checker Contract
Deterministic stale-data and error-budget policy checks are enforced through a bounded dashboard evidence lane:

- Lane command:
  - `bash scripts/dashboard/run_dashboard_stale_error_budget_lane.sh --output-json /tmp/dashboard-stale-error-report.json`
- Policy checker command:
  - `bash scripts/dashboard/check_dashboard_stale_error_budget_policy.sh --report-file /tmp/dashboard-stale-error-report.json`
- Contract lane command:
  - `bash scripts/dashboard/run_dashboard_stale_error_budget_contract_lane.sh --output-file /tmp/dashboard-stale-error-contract-report.json`

Runtime budget controls:

- `KAMN_DASHBOARD_STALE_ERROR_MAX_SECONDS`
- `KAMN_DASHBOARD_STALE_ERROR_CONTRACT_MAX_SECONDS`

Required schema/reason markers:

- `kamn.dashboard.stale-error-budget-report.v1`
- `dashboard_stale_error_budget_reason_codes:GO:v1`
- `dashboard_stale_error_budget_reason_codes:NO-GO:v1`

The lane fails closed: stale threshold drift, error-budget threshold drift, docs parity drift, or runtime budget overflow force `NO-GO` (`Regression: #942`).

## Moderation and Recovery Observability Hooks (Issue #924)
Reputation moderation actions publish deterministic quarantine/recovery evidence so operator dashboards can audit why signals were held or penalties reversed.

- Quarantine observability lane:
  - `bash scripts/reputation/run_reputation_signal_quarantine_contract_lane.sh`
  - emits deterministic `reason_key`, `reason_codes`, and `ingestion_action` fields.
- Recovery observability lane:
  - `bash scripts/reputation/run_reputation_recovery_contract_lane.sh`
  - emits deterministic `reason_key`, `reason_codes`, and `recovery_action` fields.
- Regression policy:
  - quarantined stale/replayed signals and irreversible recovery reversals must remain visible through deterministic evidence keys (`Regression: #924`).

## Local Validation
Run from repository root:

```bash
bash scripts/reputation/test_run_reputation_signal_quarantine_contract_lane.sh
bash scripts/reputation/test_run_reputation_recovery_contract_lane.sh
bash scripts/canary/test_generate_post_cutover_slo_evidence_bundle.sh
bash scripts/canary/test_run_post_cutover_slo_contract_lane.sh
cargo test -p kamn-core --test observability_stack
npm --prefix packages/kamn-dashboard test
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
