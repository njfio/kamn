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

## Local Kolme Setup For E2E Harness

For local `kamn-e2e-harness` live runs, use an upstream-supported Kolme API profile from this repo
without modifying the Kolme source tree:

```bash
git clone https://github.com/fpco/kolme /tmp/kolme
cd /tmp/kolme
RUSTFLAGS="-C link-arg=-fuse-ld=bfd" cargo build --release -p example-p2p
/tmp/kolme/target/release/example-p2p api-server --bind 127.0.0.1:3000
```

Readiness check:

```bash
curl --fail --silent --show-error http://127.0.0.1:3000/healthz
```

### Full Local E2E Smoke (Kolme + 3 KAMN Nodes + Harness Run/Verify)

Start three local KAMN API nodes (processor/listener/approver) in separate shells:

```bash
target/debug/kamn-node \
  --runtime-mode api \
  --role processor \
  --api-bind 127.0.0.1:8080 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-processor
```

```bash
target/debug/kamn-node \
  --runtime-mode api \
  --role listener \
  --api-bind 127.0.0.1:8081 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-listener
```

```bash
target/debug/kamn-node \
  --runtime-mode api \
  --role approver \
  --api-bind 127.0.0.1:8082 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-approver
```

Run harness `run` in live external-execution mode:

```bash
export KAMN_E2E_SDK_DIRECT_LIVE=true
export KAMN_ENDPOINT=http://127.0.0.1:8080
export KAMN_KOLME_ENDPOINT=http://127.0.0.1:3000
export KAMN_E2E_EXTERNAL_KAMN_PROCESSOR_BINARY=/abs/path/to/target/debug/kamn-node
export KAMN_E2E_EXTERNAL_KAMN_LISTENER_BINARY=/abs/path/to/target/debug/kamn-node
export KAMN_E2E_EXTERNAL_KAMN_APPROVER_BINARY=/abs/path/to/target/debug/kamn-node

target/debug/kamn-e2e-harness run \
  --mode sdk-direct \
  --kolme-binary /tmp/kolme/target/release/example-p2p \
  --enable-external-execution \
  --evidence-dir /tmp/kamn-e2e-live-evidence \
  --scenarios S-01 > /tmp/kamn-e2e-live-run.json
```

Run harness `verify` against generated evidence:

```bash
target/debug/kamn-e2e-harness verify \
  --evidence-dir /tmp/kamn-e2e-live-evidence \
  --kolme-chain-dump /tmp/kamn-e2e-live-evidence/kolme_chain_dump.json \
  --output /tmp/kamn-e2e-live-verify-report.json > /tmp/kamn-e2e-live-verify.json
```

Note: keep `run` and `verify` stdout files outside the evidence directory. `verify` validates
all JSON artifacts in `--evidence-dir` except manifest/chain-dump/report output and will fail
closed if unrelated JSON files are present there without `_verification` markers.

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
