# Failure Drills (Story #2980)

This document defines the implementation and validation protocol for network partition, signer incident, and finality fault drills.

## Core Lane (Task #2981 / Subtask #2982)

Composite lane artifacts:

- Runtime wrapper: `scripts/runtime/run_network_signer_finality_failure_drills_lane.sh`
- Runtime implementation: `scripts/runtime/run_network_signer_finality_failure_drills_lane_impl.sh`
- Runtime contract: `scripts/runtime/network_signer_finality_failure_drills_lane_contract.py`
- Runtime harness test: `scripts/runtime/test_run_network_signer_finality_failure_drills_lane.sh`
- Manifest: `scripts/framework/manifests/runtime_network_signer_finality_failure_drills_lane.json`

The lane composes:

1. Network partition/reconnect drill:
   - `scripts/runtime/run_live_network_partition_reconnect_smoke_lane.sh`
2. Signer incident recovery drill:
   - `scripts/signer/run_signer_incident_recovery_lane.sh`
3. Finality evidence drill:
   - `scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh`

## Commands

Run core harness:

```bash
bash scripts/runtime/test_run_network_signer_finality_failure_drills_lane.sh
```

Run baseline drill:

```bash
bash scripts/runtime/run_network_signer_finality_failure_drills_lane.sh --output-json /tmp/failure-drills.json
```

Run injected signer fault drill (expected fail-closed):

```bash
bash scripts/runtime/run_network_signer_finality_failure_drills_lane.sh --fault-profile signer --output-json /tmp/failure-drills-fault.json
```

## Deterministic Markers

Baseline GO markers:

- `status=pass`
- `final_decision=GO`
- `network_partition_status=verified`
- `signer_fault_status=verified`
- `finality_fault_status=verified`

Injected fail-closed marker:

- signer fault profile emits `signer_fault_injection_triggered` and returns non-zero.

## Evidence Schema

`kamn.runtime.failure-drills-report.v1`

Includes:

- status/final decision
- selected fault profile
- contract status markers
- reason codes
- runtime budget and elapsed duration

## Live Validation (Task #2983 / Subtask #2984)

Live validation artifacts:

- `scripts/runtime/validate_failure_drills_live.sh`
- `scripts/runtime/test_validate_failure_drills_live.sh`

Run validation harness:

```bash
bash scripts/runtime/test_validate_failure_drills_live.sh
```

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `baseline_contract_status=verified`
- `fault_injection_status=verified`
- `fail_closed_status=verified`

Injected fail-closed drill:

- signer fault profile:
  - `bash scripts/runtime/run_network_signer_finality_failure_drills_lane.sh --fault-profile signer --max-seconds 180 --partition-max-seconds 60 --signer-max-seconds 60`
  - deterministic reason marker: `signer_fault_injection_triggered`
