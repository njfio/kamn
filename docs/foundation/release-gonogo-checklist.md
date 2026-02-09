# Release Go/No-Go Checklist and Dry-Run Workflow (Issues #172, #173)

This checklist defines deterministic release gates and auditable evidence requirements before approving a protocol or runtime upgrade.
For semantic versioning policy and compatibility rules, see `docs/foundation/versioning-compatibility-matrix.md`.

## Preflight Gates
- Migration plan reviewed and signed.
- Compatibility matrix validated.
- Deployment topology preflight passed (`scripts/deploy/preflight_topology.sh`).
- CI fast gate and deferred deep lane both green.
- Rollback runbook version pinned.
- Release candidate artifact digest verified.

## Deterministic Dry-Run Workflow
1. Create release candidate tag.
2. Rehearse migration on staging snapshot.
3. Execute bounded smoke and invariant suites.
4. Capture and sign dry-run evidence bundle.
5. Validate rollback precheck against last known-good snapshot.

## Go/No-Go Evidence Template
- Release candidate:
- Schema target version:
- Runtime image digest:
- Dry-run timestamp:
- CI evidence links:
- Rollback trigger status:
- Rollback precheck result: PASS
- Final decision: GO | NO-GO
- Approver signatures:

## Machine-Readable Evidence Bundle Contract (Issue #644)
Go/no-go decisions are captured as machine-readable JSON so release policy checks are auditable and deterministic.

- Generator:
  - `bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo.json --release-candidate v1.0.0-rc.1 --schema-target-version 1.0.0 --runtime-image-digest sha256:abc123 --ci-fast-gate PASS --ci-deep-lane PASS --rollback-precheck PASS --rollback-trigger-status CLEAR --required-approvals 2 --received-approvals 2`
- Policy checker:
  - `bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo.json`
- Fast contract lane:
  - `bash scripts/deploy/run_gonogo_evidence_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/deploy/run_gonogo_evidence_deep_lane.sh`

## Staging Deploy + Rollback Rehearsal Contract (Issue #658)
Staging rehearsal automation must verify deploy and rollback outcomes before release decisions are accepted.

- Rehearsal bundle generator:
  - `bash scripts/deploy/generate_staging_rehearsal_bundle.sh --output-file /tmp/staging-rehearsal.json --release-candidate v1.1.0-rc.1 --deploy-status PASS --rollback-status PASS --rollback-target-hash state-hash-expected --post-rollback-hash state-hash-expected --evidence-complete true --ci-fast-gate PASS`
- Rehearsal policy checker:
  - `bash scripts/deploy/check_staging_rehearsal_policy.sh --bundle-file /tmp/staging-rehearsal.json`
- Fast contract lane:
  - `bash scripts/deploy/run_staging_rehearsal_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/deploy/run_staging_rehearsal_deep_lane.sh`
- Regression policy:
  - rollback target hash mismatch and incomplete rehearsal evidence force `NO-GO` (`Regression: #623`).

## Durable Guard Migration + Recovery Matrix Evidence (Issue #691)
Durable guard schema evolution and restart invariants must be proven before a release is approved.

- PR fast contract lane:
  - `bash scripts/guard/run_durable_guard_recovery_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/guard/run_durable_guard_recovery_deep_lane.sh`
- Required evidence:
  - schema mismatch errors are explicit for delivery and channel policy snapshots.
  - replay/nonce and retention invariants hold after restart recovery.
  - corrupted snapshot fixtures fail closed (`Regression: #679`).
  - PR budget check passes via `performance_durable_guard_recovery_contract_lane_budget`.
  - durable bundle store contract checks pass via `durable_guard_snapshot_store` and `performance_bundle_contract_lane_budget`.
  - nightly deep matrix executes `performance_durable_guard_recovery_matrix_deep_lane`.
  - nightly deep bundle store stress executes `performance_bundle_store_deep_lane_stress`.

## Settlement Reconciliation Evidence Contract (Issue #687)
Escrow settlement outcomes require deterministic receipt/finality evidence before release approval.

- Evidence bundle generator:
  - `bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --timeout-elapsed false --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh --bundle-file /tmp/settlement-evidence.json`
- PR fast contract lane:
  - `bash scripts/escrow/run_settlement_reconciliation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh --output-json settlement-reconciliation-report.json`
- Race matrix runner:
  - `python3 scripts/escrow/run_settlement_reconciliation_race_matrix.py --fixture fixtures/escrow_reconciliation/finality_race_cases.json --output-json settlement-reconciliation-report.json`
- Regression policy:
  - missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`).
  - timeout-before-finality pending receipts and failed receipts force `NO-GO` (`Regression: #678`).

## Mainnet Cutover Manifest Validation Contract (Issue #707)
Mainnet cutover requires deterministic triadic checkpoint manifests with explicit approval and dependency evidence.

- Schema contract:
  - `fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json`
- Validator:
  - `python3 scripts/cutover/validate_mainnet_cutover_manifest.py --manifest fixtures/mainnet_cutover/mainnet_cutover_manifest.valid.json --output-json /tmp/mainnet-cutover-validation-report.json`
- PR fast contract lane:
  - `bash scripts/cutover/run_mainnet_cutover_contract_lane.sh`
- Regression policy:
  - unresolved/non-prior dependencies and insufficient approvals force `NO-GO` (`Regression: #705`).

## Local Validation
Run from repository root:

```bash
bash scripts/cutover/test_validate_mainnet_cutover_manifest.sh
bash scripts/cutover/test_run_mainnet_cutover_contract_lane.sh
bash scripts/escrow/test_generate_settlement_reconciliation_evidence_bundle.sh
bash scripts/escrow/test_run_settlement_reconciliation_contract_lane.sh
bash scripts/escrow/test_run_settlement_reconciliation_race_matrix.sh
bash scripts/guard/test_run_durable_guard_recovery_contract_lane.sh
bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh
bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh
bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh
bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh
cargo test -p kamn-core --test mainnet_cutover_runbook_docs
cargo test -p kamn-core --test release_gonogo_checklist_docs
cargo test -p kamn-core --test durable_guard_recovery_matrix
cargo test -p kamn-core --test durable_guard_snapshot_store
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
