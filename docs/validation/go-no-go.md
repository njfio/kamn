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

Run baseline dry-run gate (CI-safe schema + policy validation only):

```bash
bash scripts/runtime/run_go_no_go_gate_lane.sh --mode dry-run --output-json /tmp/go-no-go-gate-dry-run.json
```

Run local release-candidate aggregation gate (executes artifact lanes, local-only):

```bash
KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/runtime/run_go_no_go_gate_lane.sh --mode run --output-json /tmp/go-no-go-gate-run.json
```

Run injected decision-fault profile (expected fail-closed):

```bash
bash scripts/runtime/run_go_no_go_gate_lane.sh --fault-profile gate_decision --output-json /tmp/go-no-go-gate-fault.json
```

## Deterministic Markers

Baseline GO markers:

- `status=pass`
- `final_decision=GO`
- `lane_mode=dry-run`
- `run_mode_command_status=dry_run_no_commands_executed`
- `ci_fast_gate_eligible=true`
- `ci_fast_gate_scope=ci-fast-gate`
- `fast_gate_exclusion_status=verified`
- `fast_gate_exclusion_reason_code=go_no_go_gate_run_mode_excluded_from_fast_gate`
- `go_no_go_evidence_status=dry_run_pending`
- `rollback_readiness_status=dry_run_pending`
- `dr_readiness_status=dry_run_pending`

Run-mode GO markers:

- `lane_mode=run`
- `run_mode_command_status=executed`
- `ci_fast_gate_eligible=false`
- `ci_fast_gate_scope=local-only`
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

Live validation artifacts:

- `scripts/runtime/validate_go_no_go_gate_live.sh`
- `scripts/runtime/test_validate_go_no_go_gate_live.sh`

Run validation harness:

```bash
bash scripts/runtime/test_validate_go_no_go_gate_live.sh
```

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `baseline_contract_status=verified`
- `fault_injection_status=verified`
- `fail_closed_status=verified`

Injected fail-closed drill:

- decision fault profile:
  - `bash scripts/runtime/run_go_no_go_gate_lane.sh --fault-profile gate_decision --max-seconds 120`
  - deterministic reason marker: `gate_decision_fault_injection_triggered`
