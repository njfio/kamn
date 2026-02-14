# Live Validation Environment (Phase 6.4)

This document defines the first implementation slice for Story #2975 and Task #2976.

## Objective

Provide a repeatable local live-validation environment that verifies:

- multi-process deployment topology contracts, and
- Kolme connectivity contract orchestration readiness.

The lane is designed to be fast and cost-aware by default:

- `dry-run` is the default mode.
- `run` mode requires explicit local-only opt-in (`KAMN_KOLME_LOCAL_HEAVY=1`).

## Lane Artifacts

- Runtime wrapper: `scripts/runtime/run_live_validation_environment_lane.sh`
- Runtime implementation: `scripts/runtime/run_live_validation_environment_lane_impl.sh`
- Runtime contract runner: `scripts/runtime/live_validation_environment_lane_contract.py`
- Runtime lane test harness: `scripts/runtime/test_run_live_validation_environment_lane.sh`
- Manifest: `scripts/framework/manifests/runtime_live_validation_environment_lane.json`

## What The Lane Validates

1. Multi-process topology contract:
   - `scripts/deploy/validate_deployment_assets_live.sh`
2. Kolme connectivity bundle contract:
   - `scripts/kolme/run_local_live_node_validation_bundle_lane.sh`

Both checks must pass for a GO decision.

## Commands

Run the lane test harness:

```bash
bash scripts/runtime/test_run_live_validation_environment_lane.sh
```

Run lane in default dry-run mode:

```bash
bash scripts/runtime/run_live_validation_environment_lane.sh --mode dry-run --output-json /tmp/live-validation-environment.json
```

Run lane with explicit local-heavy opt-in:

```bash
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/runtime/run_live_validation_environment_lane.sh --mode run --output-json /tmp/live-validation-environment.json
```

## Deterministic Markers

Success markers:

- `status=pass`
- `final_decision=GO`
- `topology_contract_status=verified`
- `kolme_connectivity_contract_status=verified`
- `fail_closed_status=verified`

Fail-closed markers:

- Invalid runtime budget:
  - `--max-seconds nope`
  - `KAMN_LIVE_VALIDATION_ENV_MAX_SECONDS must be an integer`
- Missing local-only opt-in for run mode:
  - `run mode requires explicit local-only opt-in via KAMN_KOLME_LOCAL_HEAVY=1`

## Evidence Schema

`kamn.runtime.live-validation-environment-report.v1`

Includes:

- lane mode
- total runtime budget and elapsed time
- contract statuses
- executed command list

## Live Validation Evidence

Task and subtask:

- Task: #2978
- Subtask: #2979

Validation lane:

- `scripts/runtime/validate_live_validation_environment_live.sh`
- `scripts/runtime/test_validate_live_validation_environment_live.sh`

Run validation harness:

```bash
bash scripts/runtime/test_validate_live_validation_environment_live.sh
```

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `lane_contract_status=verified`
- `evidence_bundle_status=verified`
- `fail_closed_status=verified`

Live fail-closed drill:

- Run-mode without local opt-in:
  - `bash scripts/runtime/run_live_validation_environment_lane.sh --mode run --max-seconds 120 --topology-max-seconds 60 --kolme-max-seconds 120`
  - deterministic reason marker: `run mode requires explicit local-only opt-in via KAMN_KOLME_LOCAL_HEAVY=1`
