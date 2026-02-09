# Mainnet Cutover Manifest and Checkpoint Validation Runbook (Issue #707)

This runbook defines a deterministic manifest contract for triadic mainnet cutover checkpoints so launch progression is auditable and fail-closed.

## Manifest Schema Contract
Cutover manifests use schema `kamn.mainnet-cutover.manifest.v1`.

- Schema reference:
  - `fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json`
- Required top-level fields:
  - `schema_version`
  - `manifest_id`
  - `release_candidate`
  - `quorum_policy`
  - `checkpoints`
- Checkpoint contract fields:
  - `id`, `order`, `role`, `status`, `approvals_required`, `approvals_received`, `approved_by`, `depends_on`, `rollback_ready`
- Role and status enums:
  - roles: `processor`, `listener`, `approver`
  - statuses: `PENDING`, `READY`, `COMPLETED`, `FAILED`

## Checkpoint Validator Contract
The validator enforces ordering, dependency, quorum, and approval evidence constraints and emits machine-readable output suitable for CI/release gate consumption.

- Validator command:
  - `python3 scripts/cutover/validate_mainnet_cutover_manifest.py --manifest fixtures/mainnet_cutover/mainnet_cutover_manifest.valid.json --output-json /tmp/mainnet-cutover-validation-report.json`
- Deterministic outputs:
  - report schema: `kamn.mainnet-cutover.validation-report.v1`
  - decision output: `validation_decision=GO` for valid manifests
  - failed validations write `decision=NO-GO` with explicit errors

## Fast Contract Lane
- Contract lane entrypoint:
  - `bash scripts/cutover/run_mainnet_cutover_contract_lane.sh`
- Coverage:
  - valid manifest acceptance path
  - non-prior/invalid dependency rejection
  - insufficient approval evidence rejection

## Cutover Rollback Evidence Contract (Issue #708)
- Evidence bundle generator:
  - `bash scripts/cutover/generate_cutover_rollback_evidence_bundle.sh --output-file /tmp/cutover-rollback.json --cutover-manifest-id cutover-mainnet-2026-02-09 --rollback-trigger-status CLEAR --checkpoint-state READY --failed-checkpoint-id '' --rollback-target-hash state-hash-abc --post-rollback-hash state-hash-abc --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/cutover/check_cutover_rollback_evidence_policy.sh --bundle-file /tmp/cutover-rollback.json`
- Fast contract lane:
  - `bash scripts/cutover/run_cutover_rollback_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/cutover/run_cutover_rollback_deep_lane.sh --output-json /tmp/cutover-rollback-report.json`

## Live-Network Pilot Cutover Evidence Gates (Issue #830)
Cutover progression requires pilot smoke and deep-lane evidence checks before checkpoint advancement.

- Pilot smoke evidence:
  - `bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json`
- Pilot deep evidence:
  - `bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name schedule --output-json /tmp/live-network-pilot-report.json`
- Pilot policy checker:
  - `bash scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh --summary-file /tmp/live-network-pilot-report.json`

## Regression Policy
- out-of-order or unresolved checkpoint dependencies force `NO-GO` (`Regression: #705`).
- missing or insufficient checkpoint approvals force `NO-GO` (`Regression: #705`).
- missing failed-checkpoint rollback evidence and rollback-target hash mismatch force `NO-GO` (`Regression: #708`).
- pilot cutover progression is blocked when smoke/deep pilot evidence is missing or policy validation is `NO-GO` (`Regression: #830`).

## Local Validation
Run from repository root:

```bash
bash scripts/cutover/test_validate_mainnet_cutover_manifest.sh
bash scripts/cutover/test_run_mainnet_cutover_contract_lane.sh
bash scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh
bash scripts/cutover/test_run_cutover_rollback_contract_lane.sh
cargo test -p kamn-core --test mainnet_cutover_runbook_docs
cargo test -p kamn-core --test release_gonogo_checklist_docs
cargo fmt --check
cargo clippy -- -D warnings
```
