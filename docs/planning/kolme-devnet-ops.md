# Kolme Triadic Devnet Operability Plan (Issues #784, #785)

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

## Regression Guard

- Marker drift remains fail-closed via fixture-backed validation (`Regression: #785`).

## Local Validation

```bash
bash scripts/kolme/test_validate_triadic_devnet_smoke.sh
bash scripts/kolme/test_run_triadic_devnet_smoke_contract_lane.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_workflow_scope_policy.sh
```
