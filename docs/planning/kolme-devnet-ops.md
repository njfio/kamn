# Kolme Triadic Devnet Operability Plan (Issues #784, #785, #787, #788, #1405, #1417, #1418)

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

## Runtime Commit Adapter Replay/Finality Fast Lane (Issue #980)

- Adapter replay/finality contract lane:
  - `bash scripts/kolme/run_runtime_commit_adapter_contract_lane.sh`
- Reason-code checks:
  - `receipt_provider_mismatch`
  - `receipt_not_final`
- Cost policy:
  - lane enforces a 60-second fast-gate budget and runs only targeted adapter/replay checks.

## Runtime Commit Block Fallback Reconciliation Fast Lane (Issue #1464)

- Block fallback reconciliation contract lane:
  - `bash scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh`
- Targeted rust fallback reconciliation test:
  - `cargo test -p kamn-core --test kolme_runtime_commit_block_fallback`
- Cost policy:
  - lane enforces a 75-second fast-gate budget via `KAMN_KOLME_BLOCK_FALLBACK_MAX_SECONDS`.

## Deterministic Local Fork Sync Metadata Lane (Issue #1429)

- Local fork metadata sync runner:
  - `bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-sync-metadata-summary.json`
- Local fork metadata validation:
  - `bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --output-json /tmp/kolme-local-fork-sync-metadata-summary.json`
- Summary schema:
  - `kamn.kolme.local-fork-sync-metadata-summary.v1`
- Deterministic checks include:
  - checkout path exists
  - checkout is a git work tree
  - `origin` remote URL matches expected fork repository
  - symbolic HEAD ref matches expected ref
  - HEAD commit is non-empty
  - checkout dirty-state guard remains fail-closed unless explicit `--allow-dirty` is set

## Bounded Local Fork Smoke Evidence Lane (Issue #1430)

- Local fork smoke evidence runner:
  - `bash scripts/kolme/run_local_fork_smoke_evidence_lane.sh --mode dry-run --checkout-path /tmp/kolme_fork --output-json /tmp/kolme-local-fork-smoke-evidence-summary.json`
- Explicit local-only smoke execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_fork_smoke_evidence_lane.sh --mode run --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --smoke-command "cargo test -p merkle-map --test version -- --exact load_from_zero_example" --max-seconds 120 --output-json /tmp/kolme-local-fork-smoke-evidence-summary.json`
- Summary schema:
  - `kamn.kolme.local-fork-smoke-evidence-summary.v1`
- Deterministic checkpoints include:
  - `run_local_fork_sync_metadata_lane.sh` metadata validation
  - bounded smoke command execution with timeout budget guard
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - smoke command timeout/exceeded budget is reported as `fork_smoke_command_timeout`.

## Deterministic Local Kolme API Probe Lane (Issue #1439)

- Local API probe runner:
  - `bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode dry-run --base-url http://127.0.0.1:3000 --output-json /tmp/kolme-local-api-probe-summary.json`
- Active local API probe:
  - `bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --max-seconds 30 --output-json /tmp/kolme-local-api-probe-summary.json`
- Summary schema:
  - `kamn.kolme.local-api-probe-summary.v1`
- Deterministic checks include:
  - `GET /healthz` response body matches expected health marker (`Healthy!` by default).
  - `GET /fork-info` returns valid JSON object with integer `first_block` and `last_block`.
- Cost policy:
  - run mode enforces a deterministic runtime budget ceiling via `--max-seconds`.

## Bounded Local-Only Kolme API Smoke Lane (Issue #1440)

- Local API smoke runner:
  - `bash scripts/kolme/run_local_kolme_api_smoke_lane.sh --mode dry-run --base-url http://127.0.0.1:3000 --smoke-command "curl --silent --show-error --fail http://127.0.0.1:3000/healthz" --output-json /tmp/kolme-local-api-smoke-summary.json`
- Explicit local-only API smoke execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kolme_api_smoke_lane.sh --mode run --base-url http://127.0.0.1:3000 --smoke-command "curl --silent --show-error --fail http://127.0.0.1:3000/healthz" --max-seconds 60 --output-json /tmp/kolme-local-api-smoke-summary.json`
- Summary schema:
  - `kamn.kolme.local-api-smoke-summary.v1`
- Deterministic checkpoints include:
  - `run_local_kolme_api_probe_lane.sh` prerequisite run-mode verification.
  - bounded smoke command execution with timeout budget guard.
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - smoke command timeout/exceeded budget is reported as `smoke_command_timeout`.

## Local Runtime Commit Live Proof Lane (Issue #1450)

- Local runtime-commit live lane runner:
  - `bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode dry-run --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt`
- Explicit opt-in live execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run --live-command "cargo test -p kamn-core --test kolme_runtime_commit_http_transport" --max-seconds 90 --output-json /tmp/kolme-local-runtime-commit-live-summary.json --live-output-file /tmp/kolme-local-runtime-commit-live-output.txt`
- Summary schema:
  - `kamn.kolme.local-runtime-commit-live-summary.v1`
- Deterministic checkpoints include:
  - explicit local-only opt-in marker (`KAMN_KOLME_LOCAL_HEAVY=1`)
  - bounded live command timeout via `--max-seconds`
  - machine-readable pass/fail reason codes for missing opt-in, command failure, and timeout
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.
  - live command timeout/exceeded budget is reported as `live_runtime_commit_command_timeout`.

## Deterministic Local Bootstrap Health Checks (Issue #1417)

- Bootstrap health-check runner:
  - `bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode dry-run --output-json /tmp/kolme-local-bootstrap-summary.json`
- Explicit opt-in bootstrap execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_bootstrap_health_checks.sh --mode run --output-json /tmp/kolme-local-bootstrap-summary.json`
- Summary schema:
  - `kamn.kolme.local-bootstrap-summary.v1`
- Deterministic readiness checks include:
  - `validate_version_compatibility.py`
  - `generate_fork_compatibility_evidence.py`
  - `check_fork_compatibility_policy.py`
  - `run_triadic_devnet_smoke.sh`
  - `validate_triadic_devnet_smoke.py`
- Cost policy:
  - run mode fails closed without explicit local-only opt-in.

## Local-Only Heavy End-to-End Lane (Issue #1418)

- Local-only E2E lane runner:
  - `bash scripts/kolme/run_local_e2e_integration_lane.sh --mode dry-run --output-json /tmp/kolme-local-e2e-integration-summary.json`
- Explicit opt-in E2E execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_e2e_integration_lane.sh --mode run --output-json /tmp/kolme-local-e2e-integration-summary.json`
- Summary schema:
  - `kamn.kolme.local-e2e-integration-summary.v1`
- Deterministic checkpoints include:
  - `run_local_bootstrap_health_checks.sh`
  - `run_runtime_commit_adapter_contract_lane.sh`
  - `run_live_transport_parity_contract_lane.sh --languages python,typescript`
- Cost policy:
  - lane enforces explicit local-only opt-in and a deterministic runtime budget ceiling.

## Local-Only Heavy Kolme Validation Matrix (Issue #1405)

- Local-only matrix runner:
  - `bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode dry-run --output-json /tmp/kolme-local-heavy-validation-summary.json`
- Explicit opt-in execution:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_heavy_validation_matrix.sh --mode run --output-json /tmp/kolme-local-heavy-validation-summary.json`
- Summary schema:
  - `kamn.kolme.local-heavy-validation-summary.v1`
- Heavy command set includes:
  - `scripts/kolme/run_local_bootstrap_health_checks.sh`
  - `scripts/kolme/run_version_compatibility_replay_deep_lane.sh`
- Cost policy:
  - matrix execution remains local-only and is excluded from PR fast-gate workflow routing.

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
- runtime commit adapter replay/finality reason-code drift fails closed (`Regression: #980`).
- deterministic bootstrap run mode fails closed without explicit local-only opt-in (`Regression: #1417`).
- local-only heavy E2E lane run mode fails closed without explicit local-only opt-in (`Regression: #1418`).
- local-only heavy validation matrix requires explicit opt-in and remains excluded from PR fast-gate workflows (`Regression: #1405`).
- local fork metadata sync lane fails closed for checkout-path, remote-URL, ref, and dirty-checkout drift (`Regression: #1429`).
- local fork smoke evidence lane fails closed on missing local opt-in, metadata sync failure, command timeout, and smoke-command errors (`Regression: #1430`).
- local Kolme API probe lane fails closed on unavailable health endpoint, invalid fork-info payload, and runtime budget overruns (`Regression: #1439`).
- local Kolme API smoke lane fails closed without explicit local opt-in, probe prerequisite failure, smoke-command timeout, and smoke-command errors (`Regression: #1440`).
- local runtime-commit live proof lane fails closed without local opt-in and for command timeout/failure paths (`Regression: #1450`).
- block fallback stale-window and response-height drift remains fail-closed (`Regression: #1464`).
- Failover/sync budget overruns and unscheduled deep-lane execution fail closed (`Regression: #788`).

## Local Validation

```bash
bash scripts/kolme/test_validate_triadic_devnet_smoke.sh
bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh
bash scripts/kolme/test_run_local_fork_sync_metadata_lane.sh
bash scripts/kolme/test_run_local_fork_smoke_evidence_lane.sh
bash scripts/kolme/test_run_local_kolme_api_probe_lane.sh
bash scripts/kolme/test_run_local_kolme_api_smoke_lane.sh
bash scripts/kolme/test_run_local_runtime_commit_live_lane.sh
bash scripts/kolme/test_run_block_fallback_reconciliation_contract_lane.sh
bash scripts/kolme/test_run_local_bootstrap_health_checks.sh
bash scripts/kolme/test_run_local_e2e_integration_lane.sh
bash scripts/kolme/test_run_local_heavy_validation_matrix.sh
bash scripts/runtime/test_select_failover_sync_drill_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_deep_lane.sh
bash scripts/runtime/test_run_failover_sync_drill_suite.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
