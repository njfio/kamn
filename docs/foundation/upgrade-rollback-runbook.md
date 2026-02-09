# Upgrade Rollback and Post-Upgrade Verification Runbook (Issues #170, #171)

This runbook defines deterministic rollback triggers, rollback execution steps, and post-upgrade verification checks for KAMN operators.
For semantic versioning policy and compatibility rules, see `docs/foundation/versioning-compatibility-matrix.md`.

## Rollback Triggers
- State migration checksum mismatch.
- Quorum health degraded below threshold.
- Critical post-upgrade verification failure.
- Invalid runtime wiring detected for processor, listener, or approver roles.

## Rollback Procedure
1. Freeze upgrade pipeline and block new proposals.
2. Confirm rollback trigger evidence.
3. Restore last known-good state snapshot.
4. Rehydrate node roles with pinned release image.
5. Re-run migration consistency checks.
6. Resume controlled traffic.

## Post-Upgrade Verification Checklist
- App-state schema version matches expected target.
- Processor, Listener, and Approver roles report healthy wiring.
- No stale state-hash acceptance detected.
- Invariant smoke harness completes with no critical failures.
- Governance and operator permission paths remain authorized.

## Failure Simulation Scenarios
- Schema mismatch rollback drill:
  - Inject incompatible migration metadata and verify trigger fires before write-path enablement.
- Partial node upgrade divergence drill:
  - Upgrade processor only, keep listeners/approvers pinned, and verify rollback restores quorum consistency.
- Quorum degradation during upgrade drill:
  - Remove one approver during staged rollout and validate rollback gate blocks finalization.

## Watchdog Incident Response Flow
1. Confirm incident attestation severity and fingerprint.
2. Capture incident evidence payload before mitigation.
3. Execute containment actions from the approved response tier.
4. Run rollback procedure when severity is `critical`.
5. Record closure summary with deterministic incident fields.

Capture incident evidence payload with expected/observed state hash, quorum sample, and censorship delivery ratios before rollback action.

## DR Drill Evidence and Release SLO Gate Contract (Issue #660)
Release promotion requires machine-validated DR drill evidence and SLO gate checks.

- DR evidence bundle generator:
  - `bash scripts/deploy/generate_dr_evidence_bundle.sh --output-file /tmp/dr-evidence.json --drill-id dr-2026-02-08-a --recovery-rto-seconds 240 --recovery-rpo-seconds 90 --max-rto-seconds 300 --max-rpo-seconds 120 --rollback-restored true --evidence-complete true --ci-fast-gate PASS`
- SLO gate policy checker:
  - `bash scripts/deploy/check_release_slo_gates.sh --bundle-file /tmp/dr-evidence.json`
- Fast contract lane:
  - `bash scripts/deploy/run_dr_evidence_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/deploy/run_dr_evidence_deep_lane.sh`
- Regression policy:
  - missing/incomplete DR evidence and SLO threshold violations force `NO-GO` (`Regression: #623`).

## Deployment SLO Evidence and Rollback Automation Contract
Deterministic deployment SLO/rollback policy checks are enforced through a bounded deployment lane:

- Lane command:
  - `bash scripts/deploy/run_deployment_slo_rollback_lane.sh --output-json /tmp/deployment-slo-rollback-report.json`
- Policy checker command:
  - `bash scripts/deploy/check_deployment_slo_rollback_policy.sh --report-file /tmp/deployment-slo-rollback-report.json`
- Contract lane command:
  - `bash scripts/deploy/run_deployment_slo_rollback_contract_lane.sh --output-file /tmp/deployment-slo-rollback-contract-report.json`

Runtime budget controls:

- `KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS`
- `KAMN_DEPLOYMENT_SLO_ROLLBACK_CONTRACT_MAX_SECONDS`

Required schema/reason markers:

- `kamn.deploy.slo-rollback-report.v1`
- `deployment_slo_rollback_reason_codes:GO:v1`
- `deployment_slo_rollback_reason_codes:NO-GO:v1`

The lane fails closed: SLO gate drift, rollback automation evidence drift, docs parity drift, or runtime budget overflow force `NO-GO` (`Regression: #944`).

## Governance Lifecycle and Rollback Integrity Contract Lane
Governance transition safety now includes deterministic lifecycle/rollback integrity evidence to prevent illegal execution or rollback divergence.

- Lifecycle/rollback lane command:
  - `bash scripts/governance/run_governance_lifecycle_rollback_lane.sh --output-file /tmp/governance-lifecycle-rollback-report.json`
- Lifecycle/rollback policy checker:
  - `bash scripts/governance/check_governance_lifecycle_rollback_policy.sh --report-file /tmp/governance-lifecycle-rollback-report.json`
- Lifecycle/rollback contract lane:
  - `bash scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh --output-file /tmp/governance-lifecycle-rollback-contract-report.json`

Runtime budget controls:

- `KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_MAX_SECONDS`
- `KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_CONTRACT_MAX_SECONDS`
- `KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS`

Required schema/reason markers:

- `kamn.governance.lifecycle-rollback-report.v1`
- `governance_lifecycle_rollback_reason_codes:GO:v1`
- `governance_lifecycle_rollback_reason_codes:NO-GO:v1`

Regression policy:

- illegal lifecycle transitions and rollback integrity drift must fail closed (`Regression: #910`).

## Live-Network Pilot Rollback Evidence Gate (Issue #830)
Pilot rollback readiness requires deterministic deep-lane evidence and policy validation before release continuity is approved.

- Pilot deep evidence:
  - `bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name schedule --output-json /tmp/live-network-pilot-report.json`
- Pilot summary policy checker:
  - `bash scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh --summary-file /tmp/live-network-pilot-report.json`
- Pilot deep contract lane:
  - `bash scripts/runtime/run_live_network_pilot_deep_contract_lane.sh`
- Regression policy:
  - pilot rollback remains mandatory when deep-lane evidence is stale, missing, or policy-invalid (`Regression: #830`).

## Fast and Cost-Effective Watchdog Validation Lane
Run from repository root:

```bash
cargo test -p kamn-core --test runtime_watchdog_attestation_docs
cargo test -p kamn-core --test watchdog_node_docs
cargo test -p kamn-core --test upgrade_rollback_runbook_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

## Local Validation
Run from repository root:

```bash
bash scripts/deploy/test_generate_dr_evidence_bundle.sh
bash scripts/deploy/test_run_dr_evidence_contract_lane.sh
bash scripts/governance/test_run_governance_lifecycle_rollback_lane.sh
bash scripts/governance/test_check_governance_lifecycle_rollback_policy.sh
bash scripts/governance/test_run_governance_lifecycle_rollback_contract_lane.sh
cargo test -p kamn-core --test upgrade_rollback_runbook_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
