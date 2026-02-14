# Go/No-Go Gate (Story #2985)

This document defines implementation and validation protocols for the production go/no-go gate and rollback readiness evidence flow.

## Core Lane (Task #2986 / Subtask #2987)

Composite lane artifacts:

- Runtime wrapper: `scripts/runtime/run_go_no_go_gate_lane.sh`
- Runtime implementation: `scripts/runtime/run_go_no_go_gate_lane_impl.sh`
- Runtime contract: `scripts/runtime/go_no_go_gate_lane_contract.py`
- Runtime harness test: `scripts/runtime/test_run_go_no_go_gate_lane.sh`
- Manifest: `scripts/framework/manifests/runtime_go_no_go_gate_lane.json`

The lane composes:

1. Go/no-go evidence deep lane:
   - `scripts/deploy/run_gonogo_evidence_deep_lane.sh`
2. Deployment rollback readiness lane:
   - `scripts/deploy/run_deployment_slo_rollback_contract_lane.sh`
3. DR evidence readiness lane:
   - `scripts/deploy/run_dr_evidence_contract_lane.sh`

## Commands

Run core harness:

```bash
bash scripts/runtime/test_run_go_no_go_gate_lane.sh
```

Run baseline gate:

```bash
bash scripts/runtime/run_go_no_go_gate_lane.sh --output-json /tmp/go-no-go-gate.json
```

Run injected decision-fault profile (expected fail-closed):

```bash
bash scripts/runtime/run_go_no_go_gate_lane.sh --fault-profile gate_decision --output-json /tmp/go-no-go-gate-fault.json
```

## Deterministic Markers

Baseline GO markers:

- `status=pass`
- `final_decision=GO`
- `go_no_go_evidence_status=verified`
- `rollback_readiness_status=verified`
- `dr_readiness_status=verified`

Injected fail-closed marker:

- decision fault profile emits `gate_decision_fault_injection_triggered` and returns non-zero.

## Evidence Schema

`kamn.runtime.go-no-go-gate-report.v1`

Includes:

- status/final decision
- selected fault profile
- evidence and readiness contract markers
- reason codes
- runtime budget and elapsed duration

## Live Validation (Task #2988 / Subtask #2989)

Dedicated live-validation drill scripts and evidence markers are tracked in Task #2988 and Subtask #2989.
