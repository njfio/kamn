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
- Stable shell wrappers:
  - `scripts/deploy/generate_dr_evidence_bundle.sh`
  - `scripts/deploy/check_release_slo_gates.sh`
- Shared Python implementation:
  - `scripts/deploy/dr_evidence_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/deploy/run_dr_evidence_deep_lane.sh`
- Regression policy:
  - missing/incomplete DR evidence and SLO threshold violations force `NO-GO` (`Regression: #623`).

## Secure-Signer Incident Recovery Contract Lanes (Issue #989)
Signer incident response requires deterministic lane reports, fail-closed policy checks, and scheduled deep-lane cadence enforcement.

- Incident recovery lane:
  - `bash scripts/signer/run_signer_incident_recovery_lane.sh --output-json /tmp/signer-incident-recovery-report.json`
- Policy checker:
  - `bash scripts/signer/check_signer_incident_recovery_policy.sh --report-file /tmp/signer-incident-recovery-report.json`
- PR fast contract lane:
  - `bash scripts/signer/run_signer_incident_recovery_contract_lane.sh --output-file /tmp/signer-incident-recovery-contract-report.json`
- Scheduled deep lane:
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE=scheduled bash scripts/signer/run_signer_incident_recovery_deep_lane.sh --output-json /tmp/signer-incident-recovery-deep-report.json`
- Stable shell wrappers:
  - `scripts/signer/run_signer_incident_recovery_lane.sh`
  - `scripts/signer/check_signer_incident_recovery_policy.sh`
  - `scripts/signer/run_signer_incident_recovery_contract_lane.sh`
  - `scripts/signer/run_signer_incident_recovery_deep_lane.sh`
- Shared Python implementation:
  - `scripts/signer/signer_incident_recovery_lane.py`
  - `scripts/signer/signer_incident_recovery_policy_contract.py`
  - `scripts/signer/signer_incident_recovery_contract_lane_contract.py`
- Runtime and cadence controls:
  - `KAMN_SIGNER_INCIDENT_RECOVERY_MAX_SECONDS`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_CONTRACT_MAX_SECONDS`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_SECONDS`
  - `KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_MAX_ARTIFACT_AGE_SECONDS`
- Required schema/reason markers:
  - `kamn.signer.incident-recovery-report.v1`
  - `kamn.signer.incident-recovery-deep-summary.v1`
  - `signer_incident_recovery_reason_codes:GO:v1`
  - `signer_incident_recovery_deep_reason_codes:GO:v1`
- Regression policy:
  - runbook-step drift, revocation propagation gaps, stale deep-lane artifacts, or cadence violations force `NO-GO` (`Regression: #989`).

## Kolme Multi-Signer Deployment Preflight Contract Lane (Issue #2301)
Local Kolme deployment gates require deterministic multi-signer quorum and custody evidence before live runtime operations proceed.

- Deployment preflight lane command:
  - `bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
- Quorum/custody/provenance evidence preparation:
  - `printf '%s\n' "custody-attestation=ops-primary:epoch-1" > /tmp/kolme-live-signer-custody.json`
  - `printf '%s\n' "signer-provenance=ops-primary:source-managed-external:epoch-1" > /tmp/kolme-live-signer-provenance.json`
  - `custody_sha="$(sha256sum /tmp/kolme-live-signer-custody.json | awk '{print $1}')"; cat > /tmp/kolme-live-signer-quorum.json <<JSON
{
  "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
  "required_approvals": 2,
  "received_approvals": 2,
  "approved_signers": ["ops-primary", "ops-secondary"],
  "custody_evidence_sha256": "$custody_sha"
}
JSON`
- Deployment preflight run mode:
  - `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=1111111111111111111111111111111111111111111111111111111111111111 bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode run --runtime-mode kolme-live --signer-profile ops-primary --required-approvals 2 --received-approvals 2 --quorum-evidence-file /tmp/kolme-live-signer-quorum.json --custody-evidence-file /tmp/kolme-live-signer-custody.json --signer-provenance-file /tmp/kolme-live-signer-provenance.json --signer-key-source managed-external --signer-key-source-contract-version v1 --signer-rotation-epoch 3 --signer-previous-rotation-epoch 1 --signer-rotation-freshness-max-delta 2 --max-seconds 12 --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
- Deployment preflight policy checker:
  - `python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code deployment_preflight_passed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
- Deployment preflight contract lane:
  - `bash scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh --output-json /tmp/kolme-local-live-deployment-preflight-summary.json --policy-output-json /tmp/kolme-local-live-deployment-preflight-policy.json`
- Required schema/reason markers:
  - `kamn.kolme.local-live-deployment-preflight-summary.v1`
  - `kamn.kolme.local-live-deployment-preflight-policy-report.v1`
  - `kamn.kolme.signer-quorum-evidence.v1`
  - `kamn.kolme.runtime-signer-attestation.v1`
  - `runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1`
  - `runtime_signer_attestation_bundle`
  - `runtime_signer_drift_thresholds_schema_version=kamn.kolme.runtime-signer-drift-thresholds.v1`
  - `runtime_signer_drift_thresholds_bundle`
  - `runtime_signer_drift_admission_matrix_decision=GO|WARN|NO-GO`
  - `runtime_signer_drift_admission_matrix_class=healthy|warning-edge|hard-fail`
  - `runtime_signer_drift_admission_matrix_reason_codes`
  - `checkpoint_failed_signer_quorum_contract`
  - `checkpoint_failed_quorum_evidence_contract`
  - `checkpoint_failed_custody_evidence_contract`
  - `quorum_evidence_missing`
  - `quorum_evidence_custody_sha256_mismatch`
  - `runtime_signer_attestation_approved_signers_not_unique`
  - `runtime_signer_attestation_quorum_shortfall`
  - `runtime_signer_drift_quorum_fail_threshold_exceeded`
  - `runtime_signer_drift_rotation_fail_threshold_exceeded`
- Drift-breach response:
  - If matrix class is `warning-edge`, freeze promotion, rotate signer evidence within threshold, and rerun preflight lane/policy before continuing.
  - If matrix class is `hard-fail` or decision is `NO-GO`, block rollout, execute signer incident rollback flow, and require fresh custody/quorum/provenance artifacts before unfreezing.
- Regression policy:
  - signer quorum/custody/provenance drift, quorum evidence schema drift, or contract-lane/docs parity drift force `NO-GO` (`Regression: #2301`).
  - runtime/deployment signer-attestation schema and reason-code drift force `NO-GO` (`Regression: #2326`).

## Kolme Fallback Signer Runtime/Deploy Guard (Issue #2302)
Fallback private-key surfaces are forbidden in deployment preflight and runtime launch paths; checks fail closed with deterministic remediation guidance.

- Deployment preflight fallback guard:
  - `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=1111111111111111111111111111111111111111111111111111111111111111 bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode run --runtime-mode kolme-live --signer-profile ops-primary --required-approvals 2 --received-approvals 2 --custody-evidence-file /tmp/kolme-live-signer-custody.json --quorum-evidence-file /tmp/kolme-live-signer-quorum.json --signer-provenance-file /tmp/kolme-live-signer-provenance.json --signer-key-source managed-external --signer-key-source-contract-version v1 --signer-rotation-epoch 3 --signer-previous-rotation-epoch 1 --signer-rotation-freshness-max-delta 2 --max-seconds 12 --output-json /tmp/kolme-local-live-deployment-preflight-summary.json`
- Runtime integration fallback guard:
  - `KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Runtime integration managed-external raw-key guard:
  - `KAMN_KOLME_LOCAL_HEAVY=1 KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=1111111111111111111111111111111111111111111111111111111111111111 bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --runtime-profile real-node --runtime-signer-key-source managed-external --checkout-path /tmp/kolme_fork --expected-remote-url https://github.com/njfio/kolme_fork.git --expected-ref refs/heads/main --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2 --runtime-provider-client-contract KolmeRuntimeCommitLiveProvider --runtime-commit-live-summary /tmp/kolme-local-runtime-commit-live-summary.json --runtime-commit-live-policy-report /tmp/kolme-local-runtime-commit-live-policy.json --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json`
- Contract/policy coverage:
  - `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh`
  - `bash scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
  - `bash scripts/kolme/test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh`
  - `bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
- Signer key-source profile matrix validation:
  - `bash scripts/kolme/run_managed_signer_startup_live_validation_contract_lane.sh --output-json /tmp/managed-signer-startup-live-validation-contract-report.json`
  - matrix status markers:
    - `signer_key_source_profile_matrix_status=verified`
    - `signer_key_source_production_reject_status=verified`
    - `signer_key_source_local_override_allow_status=verified`
  - production strict env-local fail-closed marker:
    - `production_signer_key_source_env_local_forbidden`
  - explicit local override marker for controlled local testing:
    - `KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=true`
- Required schema/reason markers:
  - `runtime_signer_fallback_guard_contract_version=v2`
  - `runtime_signer_fallback_guard_mode=reject_if_present`
  - `runtime_signer_managed_external_raw_private_key_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX; set KAMN_KOLME_LIVE_SIGNER_KEY_REF`
  - `runtime_signer_fallback_private_key_present=false`
  - `runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF`
  - `runtime_signer_raw_private_key_present=false`
  - `runtime_signer_fallback_private_key_present_violation`
  - `runtime_signer_managed_external_raw_private_key_present_violation`
  - `contracts.runtime_signer_fallback_guard_contract_version=v2`
  - `contracts.runtime_signer_fallback_guard_mode=reject_if_present`
  - `contracts.runtime_signer_managed_external_raw_private_key_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX; set KAMN_KOLME_LIVE_SIGNER_KEY_REF`
  - `contracts.runtime_signer_fallback_private_key_allowed=false`
  - `contracts.runtime_signer_managed_external_raw_private_key_allowed=false`
- Incident response and remediation:
  - deterministic error output must identify violating source and command-level remediation:
    - `fallback signer secret env must not be set: KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK (remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK)`
    - `managed-external signer raw private key env must not be set: KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX (remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX; set KAMN_KOLME_LIVE_SIGNER_KEY_REF)`
  - runbook response order: freeze launch -> unset fallback env -> re-run deployment preflight -> re-run runtime integration lane -> archive updated evidence.
- Regression policy:
  - fallback signer key path remains fail-closed across runtime launch + wrapper/manifest entry points (`Regression: #2302`).
  - managed-external raw signer key path remains fail-closed across runtime launch + wrapper/manifest entry points (`Regression: #2324`).

## Deployment SLO Evidence and Rollback Automation Contract
Deterministic deployment SLO/rollback policy checks are enforced through a bounded deployment lane:

- Lane command:
  - `bash scripts/deploy/run_deployment_slo_rollback_lane.sh --output-json /tmp/deployment-slo-rollback-report.json`
- Policy checker command:
  - `bash scripts/deploy/check_deployment_slo_rollback_policy.sh --report-file /tmp/deployment-slo-rollback-report.json`
- Contract lane command:
  - `bash scripts/deploy/run_deployment_slo_rollback_contract_lane.sh --output-file /tmp/deployment-slo-rollback-contract-report.json`
- Stable shell wrapper:
  - `scripts/deploy/run_deployment_slo_rollback_lane.sh`
- Shared Python implementation:
  - `scripts/deploy/deployment_slo_rollback_lane_contract.py`
- Stable shell wrapper:
  - `scripts/deploy/check_deployment_slo_rollback_policy.sh`
- Shared Python implementation:
  - `scripts/deploy/deployment_slo_rollback_policy_contract.py`

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
- Stable shell wrapper:
  - `scripts/governance/run_governance_lifecycle_rollback_lane.sh`
- Shared Python implementation:
  - `scripts/governance/governance_lifecycle_rollback_lane_contract.py`
- Lifecycle/rollback policy checker:
  - `bash scripts/governance/check_governance_lifecycle_rollback_policy.sh --report-file /tmp/governance-lifecycle-rollback-report.json`
- Stable shell wrapper:
  - `scripts/governance/check_governance_lifecycle_rollback_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_lifecycle_rollback_policy_contract.py`
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
bash scripts/signer/test_run_signer_incident_recovery_lane.sh
bash scripts/signer/test_check_signer_incident_recovery_policy.sh
bash scripts/signer/test_run_signer_incident_recovery_contract_lane.sh
bash scripts/signer/test_run_signer_incident_recovery_deep_lane.sh
bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_lane.sh
bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh
bash scripts/kolme/test_run_local_kolme_live_deployment_preflight_contract_lane.sh
bash scripts/kolme/test_run_local_kamn_live_runtime_integration_real_node_profile.sh
bash scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh
bash scripts/kolme/test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh
bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh
bash scripts/governance/test_run_governance_lifecycle_rollback_lane.sh
bash scripts/governance/test_check_governance_lifecycle_rollback_policy.sh
bash scripts/governance/test_run_governance_lifecycle_rollback_contract_lane.sh
cargo test -p kamn-core --test upgrade_rollback_runbook_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
