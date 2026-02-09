# Kolme Triadic Devnet Operability Plan (Issues #784, #785, #787, #788)

This plan defines the deterministic, low-cost local smoke contract for triadic
runtime roles (processor/listener/approver) and its CI-compatible validation.

## Scope

- One-command triadic devnet smoke orchestration.
- Deterministic marker validation from fixture contract.
- PR-safe runtime budget guard for smoke lane cost control.

## Contract Commands

- Run triadic smoke orchestration:
  - `bash scripts/kolme/run_triadic_devnet_smoke.sh --output-file /tmp/triadic-devnet-markers.txt`
- Validate observed markers:
  - `python3 scripts/kolme/validate_triadic_devnet_smoke.py --fixture fixtures/kolme_compatibility/devnet_smoke_markers.json --marker-file /tmp/triadic-devnet-markers.txt --output-json /tmp/triadic-devnet-report.json`
- Run budgeted contract lane:
  - `bash scripts/kolme/run_triadic_devnet_smoke_contract_lane.sh --output-json /tmp/triadic-devnet-report.json`

## Deterministic Marker Contract

- Fixture file:
  - `fixtures/kolme_compatibility/devnet_smoke_markers.json`
- Required markers:
  - `marker_startup=ok`
  - `marker_tx_progression=ok`
  - `marker_block_commit=ok`
  - `marker_teardown=ok`
  - `status=pass`

## Runtime and Cost Policy

- PR contract lane budget:
  - `run_triadic_devnet_smoke.sh` and `run_triadic_devnet_smoke_contract_lane.sh` enforce a 180-second ceiling.
- Bounded runtime calls:
  - smoke runner executes only targeted triadic role smoke tests to avoid full-suite costs.
- CI compatibility:
  - lane is non-interactive and emits machine-readable validation report output.

## Failover + Sync Drill Lane Policy (Issues #787, #788)

- Selector policy:
  - `bash scripts/runtime/select_failover_sync_drill_lane.sh --event-name pull_request`
  - `pull_request` routes to `preflight`; `schedule` and `workflow_dispatch` route to `deep`.
- PR-fast preflight lane:
  - `bash scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh --output-json /tmp/failover-sync-preflight-report.json`
  - preflight lane enforces a bounded runtime budget (default 15 seconds).
- Scheduled deep lane:
  - `KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled bash scripts/runtime/run_failover_sync_drill_deep_lane.sh --output-json /tmp/failover-sync-deep-report.json`
  - deep lane fails closed when invoked without scheduled cadence marker.
- CI-oriented suite entrypoint:
  - `bash scripts/runtime/run_failover_sync_drill_suite.sh --event-name schedule --output-json /tmp/failover-sync-suite-report.json`
  - suite report schema: `kamn.runtime.failover-sync-drill-suite-report.v1`.

## Regression Guard

- Marker drift remains fail-closed via fixture-backed validation (`Regression: #785`).
- Failover/sync budget overruns and unscheduled deep-lane execution fail closed (`Regression: #788`).

## Local Validation

```bash
bash scripts/kolme/test_validate_triadic_devnet_smoke.sh
bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh
bash scripts/runtime/test_select_failover_sync_drill_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_deep_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_suite.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
