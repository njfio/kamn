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
  - `bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --ledger-reference-id ledger-entry-001 --timeout-elapsed false --ci-fast-gate PASS`
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
  - missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).

## SOC2 Control Evidence Contract (Issue #744)
SOC2 audit gates require deterministic control-evidence bundles and replay-safe checker outcomes before release progression.

- Stable shell wrappers:
  - `scripts/compliance/generate_soc2_control_evidence_bundle.sh`
  - `scripts/compliance/check_soc2_control_evidence_policy.sh`
- Shared Python implementation:
  - `scripts/compliance/soc2_control_contract.py`
- Evidence bundle generator:
  - `bash scripts/compliance/generate_soc2_control_evidence_bundle.sh --output-file /tmp/soc2-control-evidence.json --control-id CC6.1 --audit-period-start 2026-01-01 --audit-period-end 2026-01-31 --collector-did did:kamn:auditor-001 --evidence-uri s3://kamn-audit/soc2/cc6_1/jan-2026/evidence.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --tamper-check PASS --completeness-check PASS --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/compliance/check_soc2_control_evidence_policy.sh --bundle-file /tmp/soc2-control-evidence.json`
- PR fast contract lane:
  - `bash scripts/compliance/run_soc2_control_evidence_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/compliance/run_soc2_control_evidence_deep_lane.sh --output-json soc2-control-evidence-report.json`
- Replay matrix runner:
  - `python3 scripts/compliance/run_soc2_control_evidence_replay_matrix.py --fixture fixtures/compliance_soc2/control_evidence_replay_cases.json --output-json soc2-control-evidence-report.json`
- Regression policy:
  - tampered final decisions and incomplete/tampered control evidence force `NO-GO` (`Regression: #732`).

## DSAR Legal-Hold Evidence Contract (Issue #746)
GDPR data-subject workflows require deterministic legal-hold precedence evidence before export/erasure approvals.

- Stable shell wrappers:
  - `scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh`
  - `scripts/compliance/check_dsar_legal_hold_policy.sh`
- Shared Python implementation:
  - `scripts/compliance/dsar_legal_hold_contract.py`
- Evidence bundle generator:
  - `bash scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh --output-file /tmp/dsar-legal-hold.json --request-id dsar-erasure-001 --subject-did did:kamn:subject-001 --request-type ERASURE --legal-hold-active true --retention-expired true --evidence-complete true --approval-recorded true --tamper-check PASS --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/compliance/check_dsar_legal_hold_policy.sh --bundle-file /tmp/dsar-legal-hold.json`
- PR fast contract lane:
  - `bash scripts/compliance/run_dsar_legal_hold_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/compliance/run_dsar_legal_hold_deep_lane.sh --output-json dsar-legal-hold-report.json`
- Replay matrix runner:
  - `python3 scripts/compliance/run_dsar_legal_hold_matrix.py --fixture fixtures/compliance_dsar/legal_hold_precedence_cases.json --output-json dsar-legal-hold-report.json`
- Regression policy:
  - legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`).

## Federated DID Handshake Evidence Contract (Issue #752)
Federated DID trust handshakes require deterministic replay, downgrade, and quorum evidence before cross-network approval.

- Evidence bundle generator:
  - `bash scripts/did/generate_federated_did_handshake_evidence_bundle.sh --output-file /tmp/federated-did-handshake.json --handshake-id federated-go-001 --subject-did kamn:did:agent:federated-worker-1 --local-network kolme-mainnet-a --remote-network kolme-mainnet-b --resolver-cache-hit true --resolver-version resolver-v1 --signature-policy PASS --nonce-monotonic true --downgrade-detected false --partition-sequence-monotonic true --required-quorum 2 --received-quorum 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/did/check_federated_did_handshake_policy.sh --bundle-file /tmp/federated-did-handshake.json`
- PR fast contract lane:
  - `bash scripts/did/run_federated_did_handshake_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/did/run_federated_did_handshake_deep_lane.sh --output-json federated-did-handshake-report.json`
- Partition replay matrix runner:
  - `python3 scripts/did/run_federated_did_handshake_matrix.py --fixture fixtures/federated_did_handshake/partition_replay_cases.json --output-json federated-did-handshake-report.json`
- Regression policy:
  - replay/downgrade attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`).

## Federated Delegation Settlement Evidence Contract (Issue #754)
Cross-network task delegation requires deterministic envelope and settlement reference evidence before cross-network approvals.

- Evidence bundle generator:
  - `bash scripts/task/generate_federated_delegation_settlement_evidence_bundle.sh --output-file /tmp/federated-delegation-settlement.json --delegation-id delegation-go-001 --task-id task-go-001 --delegator-did kamn:did:agent:delegator-go-001 --delegatee-did kamn:did:agent:delegatee-go-001 --source-network kolme-mainnet-a --destination-network kolme-mainnet-b --settlement-reference-id settlement-ref-go-001 --expected-settlement-reference-id settlement-ref-go-001 --settlement-receipt-finality FINAL --nonce-monotonic true --replay-detected false --partition-sequence-monotonic true --required-attestors 2 --received-attestors 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/task/check_federated_delegation_settlement_policy.sh --bundle-file /tmp/federated-delegation-settlement.json`
- PR fast contract lane:
  - `bash scripts/task/run_federated_delegation_settlement_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/task/run_federated_delegation_settlement_deep_lane.sh --output-json federated-delegation-settlement-report.json`
- Partition replay matrix runner:
  - `python3 scripts/task/run_federated_delegation_settlement_matrix.py --fixture fixtures/federated_task_delegation/partition_replay_cases.json --output-json federated-delegation-settlement-report.json`
- Regression policy:
  - settlement reference drift, replay attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`).

## Kolme Version Compatibility Replay Evidence Contract (Issue #780)
Kolme upgrade approvals require deterministic KAMN/Kolme version compatibility validation and replay artifact evidence.

- Version compatibility validator:
  - `python3 scripts/kolme/validate_version_compatibility.py --kamn-version 1.1.0 --kolme-release-tag v0.15.2 --ci-fast-gate PASS --output-json /tmp/kolme-version-report.json`
- Replay matrix runner:
  - `python3 scripts/kolme/run_version_compatibility_replay.py --fixture fixtures/kolme_compatibility/version_compatibility_cases.json --output-json /tmp/kolme-version-replay-report.json`
- Runtime commit replay policy checker:
  - `python3 scripts/kolme/check_runtime_commit_replay_policy.py --operation-id op-go-001 --idempotency-key kolme-runtime-commit:op-go-001:state:agent:1:12 --receipt-provider kolme-local --expected-receipt-provider kolme-local --receipt-commit-id kolme-commit:op-go-001:agent:1:12 --expected-receipt-commit-id kolme-commit:op-go-001:agent:1:12 --nonce-monotonic true --replay-detected false --payload-hash-match true --receipt-finality FINAL --ci-fast-gate PASS --output-json /tmp/kolme-runtime-commit-replay-policy.json`
- Runtime commit replay matrix runner:
  - `python3 scripts/kolme/run_runtime_commit_replay_tamper_matrix.py --fixture fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json --output-json /tmp/kolme-runtime-commit-replay-report.json`
- PR fast contract lane:
  - `bash scripts/kolme/run_version_compatibility_contract_lane.sh`
  - `bash scripts/kolme/run_runtime_commit_replay_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/kolme/run_version_compatibility_replay_deep_lane.sh --output-json kolme-version-compatibility-report.json`
- Regression policy:
  - incompatible upgrade signature (`kamn 1.2.x` + `kolme 0.14.x`) remains blocked (`Regression: #775`).
  - runtime commit replay/tamper mismatches and non-final receipts force `NO-GO` (`Regression: #827`).

## Failover + Sync Drill Evidence Contract (Issues #787, #788)
Runtime failover and sync readiness requires deterministic lane routing, budget guards, and scheduled-cadence enforcement before release approval.

- Lane selector:
  - `bash scripts/runtime/select_failover_sync_drill_lane.sh --event-name pull_request`
- PR fast contract lane:
  - `bash scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh --output-json /tmp/failover-sync-preflight-report.json`
- Scheduled deep lane entrypoint:
  - `KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled bash scripts/runtime/run_failover_sync_drill_deep_lane.sh --output-json /tmp/failover-sync-deep-report.json`
- Suite/artifact summary entrypoint:
  - `bash scripts/runtime/run_failover_sync_drill_suite.sh --event-name schedule --output-json /tmp/failover-sync-suite-report.json`
- Regression policy:
  - preflight runtime budget overruns force lane failure (`Regression: #788`).
  - unscheduled deep-lane execution force-fails via scheduled-only cadence guard (`Regression: #788`).

## Live-Network Pilot Launch and Rollback Evidence Gates (Issue #830)
Pilot launch gates require deterministic smoke and scheduled/manual deep-lane evidence before release approval.

- PR-fast smoke evidence:
  - `bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json`
- Scheduled/manual deep evidence:
  - `bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name schedule --output-json /tmp/live-network-pilot-report.json`
- Deep summary policy checker:
  - `bash scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh --summary-file /tmp/live-network-pilot-report.json`
- Contract lane:
  - `bash scripts/runtime/run_live_network_pilot_deep_contract_lane.sh`
- Regression policy:
  - missing smoke/deep pilot evidence or non-`GO` pilot decisions force launch `NO-GO` and trigger rollback review (`Regression: #830`).

## Governance Simulation and Human-Veto Evidence Contract (Issue #748)
Governance activation requires deterministic simulation, veto, timelock, and approval evidence before GO decisions.

- Stable shell wrappers:
  - `scripts/governance/generate_governance_simulation_evidence_bundle.sh`
  - `scripts/governance/check_governance_simulation_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_simulation_contract.py`
- Evidence bundle generator:
  - `bash scripts/governance/generate_governance_simulation_evidence_bundle.sh --output-file /tmp/governance-simulation.json --proposal-id gov-proposal-activation-001 --simulation-hash sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --simulation-complete true --veto-window-open false --veto-recorded false --timelock-expired true --required-approvals 2 --received-approvals 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/governance/check_governance_simulation_policy.sh --bundle-file /tmp/governance-simulation.json`
- PR fast contract lane:
  - `bash scripts/governance/run_governance_simulation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/governance/run_governance_simulation_deep_lane.sh --output-json governance-simulation-report.json`
- Replay matrix runner:
  - `python3 scripts/governance/run_governance_simulation_matrix.py --fixture fixtures/governance_simulation/veto_timelock_cases.json --output-json governance-simulation-report.json`
- Regression policy:
  - simulation/veto bypass attempts and tampered evidence bundles force `NO-GO` (`Regression: #733`).

## Governance Stake/Slash Risk Threshold Contract (Issue #750)
Governance activation requires deterministic stake/slash risk thresholds to block unsafe economic-impact scenarios.

- Evidence bundle generator:
  - `bash scripts/governance/generate_stake_slash_risk_evidence_bundle.sh --output-file /tmp/stake-slash-risk.json --proposal-id gov-risk-001 --simulation-hash sha256:1111111111111111111111111111111111111111111111111111111111111111 --stake-at-risk-bps 120 --max-stake-at-risk-bps 300 --slash-probability-bps 40 --max-slash-probability-bps 150 --validator-churn-bps 60 --max-validator-churn-bps 180 --quorum-safety-margin-bps 220 --min-quorum-safety-margin-bps 150 --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/governance/check_stake_slash_risk_policy.sh --bundle-file /tmp/stake-slash-risk.json`
- PR fast contract lane:
  - `bash scripts/governance/run_stake_slash_risk_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/governance/run_stake_slash_risk_deep_lane.sh --output-json governance-stake-slash-report.json`
- Replay matrix runner:
  - `python3 scripts/governance/run_stake_slash_risk_matrix.py --fixture fixtures/governance_stake_slash/risk_threshold_cases.json --output-json governance-stake-slash-report.json`
- Regression policy:
  - unsafe threshold bypass attempts and tampered risk evidence force `NO-GO` (`Regression: #733`).

## Reputation Dispute Evidence Contract (Issue #738)
Reputation dispute decisions require deterministic evidence bundles so trust-score corrections remain auditable and tamper-evident.

- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_dispute_evidence_bundle.sh --output-file /tmp/reputation-dispute.json --dispute-id dispute-001 --subject-did did:kamn:agent-001 --reviewer-did did:kamn:reviewer-001 --dispute-reason-code QUALITY --evidence-uri s3://kamn-audit/reputation/dispute-001.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --evidence-hash-verified PASS --original-trust-score 640 --proposed-trust-score 560 --max-adjustment-points 120 --policy-window-open true --approval-recorded true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_dispute_policy.sh --bundle-file /tmp/reputation-dispute.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_dispute_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/reputation/run_reputation_dispute_deep_lane.sh --output-json reputation-dispute-report.json`
- Replay matrix runner:
  - `python3 scripts/reputation/run_reputation_dispute_matrix.py --fixture fixtures/reputation_dispute/replay_cases.json --output-json reputation-dispute-report.json`
- Regression policy:
  - tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`).

## Token Launch Handoff Evidence Contract (Issue #714)
Token launch readiness requires deterministic supply/allocation and approval evidence before activation.

- Evidence bundle generator:
  - `bash scripts/token/generate_token_launch_handoff_evidence_bundle.sh --output-file /tmp/token-launch-handoff.json --token-symbol KAMN --configured-total-supply 1000000000 --expected-total-supply 1000000000 --configured-allocation-sum 1000000000 --expected-allocation-sum 1000000000 --allocation-bucket-count 5 --expected-bucket-count 5 --genesis-hash sha256:token-launch-handoff-go-2026-02-09 --required-approvals 2 --received-approvals 2 --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/token/check_token_launch_handoff_policy.sh --bundle-file /tmp/token-launch-handoff.json`
- PR fast contract lane:
  - `bash scripts/token/run_token_launch_handoff_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/token/run_token_launch_handoff_deep_lane.sh --output-json token-launch-handoff-report.json`
- Regression policy:
  - supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`).

## Treasury Disbursement Approval Evidence Contract (Issue #716)
Treasury disbursement execution requires deterministic approval-threshold evidence and policy-window validation.

- Evidence bundle generator:
  - `bash scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh --output-file /tmp/treasury-disbursement.json --disbursement-id disbursement-go-001 --treasury-account-id treasury-main-001 --destination-account-id ops-wallet-001 --asset-symbol KAMN --disbursement-amount 250000 --daily-limit-amount 500000 --required-approvals 2 --received-approvals 2 --approval-quorum-hash sha256:approval-go-001 --policy-window-open true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/treasury/check_treasury_disbursement_policy.sh --bundle-file /tmp/treasury-disbursement.json`
- PR fast contract lane:
  - `bash scripts/treasury/run_treasury_disbursement_contract_lane.sh`
- Regression policy:
  - insufficient approvals, approval-window closure, and daily-limit overruns force `NO-GO` (`Regression: #716`).

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

## Cutover Rollback Evidence Contract (Issue #708)
Rollback readiness and trigger execution must emit deterministic evidence before cutover approval.

- Evidence bundle generator:
  - `bash scripts/cutover/generate_cutover_rollback_evidence_bundle.sh --output-file /tmp/cutover-rollback.json --cutover-manifest-id cutover-mainnet-2026-02-09 --rollback-trigger-status CLEAR --checkpoint-state READY --failed-checkpoint-id '' --rollback-target-hash state-hash-abc --post-rollback-hash state-hash-abc --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/cutover/check_cutover_rollback_evidence_policy.sh --bundle-file /tmp/cutover-rollback.json`
- PR fast contract lane:
  - `bash scripts/cutover/run_cutover_rollback_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/cutover/run_cutover_rollback_deep_lane.sh --output-json cutover-rollback-report.json`
- Regression policy:
  - missing failed-checkpoint evidence and rollback-target hash mismatch force `NO-GO` (`Regression: #708`).

## Launch Canary Critical-Path Contract (Issue #710)
Launch approval requires deterministic critical-path probe evidence covering message/task/escrow behavior.

- Probe fixture matrix:
  - `fixtures/launch_canary/critical_path_probe_cases.json`
- Matrix runner:
  - `python3 scripts/canary/run_launch_canary_matrix.py --fixture fixtures/launch_canary/critical_path_probe_cases.json --output-json /tmp/launch-canary-report.json`
- PR fast contract lane:
  - `bash scripts/canary/run_launch_canary_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/canary/run_launch_canary_deep_lane.sh --output-json launch-canary-report.json`
- Regression policy:
  - missing probe evidence and failing critical-path probes force `NO-GO` (`Regression: #710`).

## Post-Cutover SLO Gate Evidence Contract (Issue #711)
Post-cutover launch gates require deterministic SLO evidence export with stale/partial evidence rejection.

- Evidence bundle generator:
  - `bash scripts/canary/generate_post_cutover_slo_evidence_bundle.sh --output-file /tmp/post-cutover-slo.json --window-minutes 15 --p95-latency-ms 140 --max-p95-latency-ms 200 --error-rate-bps 18 --max-error-rate-bps 25 --delivery-success-bps 9992 --min-delivery-success-bps 9950 --snapshot-age-seconds 30 --max-snapshot-age-seconds 120 --evidence-complete true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/canary/check_post_cutover_slo_policy.sh --bundle-file /tmp/post-cutover-slo.json`
- PR fast contract lane:
  - `bash scripts/canary/run_post_cutover_slo_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/canary/run_post_cutover_slo_deep_lane.sh --output-json post-cutover-slo-report.json`
- Regression policy:
  - stale snapshots and incomplete SLO evidence force `NO-GO` (`Regression: #711`).

## Local Validation
Run from repository root:

```bash
bash scripts/canary/test_run_launch_canary_matrix.sh
bash scripts/canary/test_run_launch_canary_contract_lane.sh
bash scripts/canary/test_generate_post_cutover_slo_evidence_bundle.sh
bash scripts/canary/test_run_post_cutover_slo_contract_lane.sh
bash scripts/cutover/test_validate_mainnet_cutover_manifest.sh
bash scripts/cutover/test_run_mainnet_cutover_contract_lane.sh
bash scripts/cutover/test_generate_cutover_rollback_evidence_bundle.sh
bash scripts/cutover/test_run_cutover_rollback_contract_lane.sh
bash scripts/escrow/test_generate_settlement_reconciliation_evidence_bundle.sh
bash scripts/escrow/test_run_settlement_reconciliation_contract_lane.sh
bash scripts/escrow/test_run_settlement_reconciliation_race_matrix.sh
bash scripts/compliance/test_generate_soc2_control_evidence_bundle.sh
bash scripts/compliance/test_run_soc2_control_evidence_contract_lane.sh
bash scripts/compliance/test_run_soc2_control_evidence_replay_matrix.sh
bash scripts/compliance/test_run_soc2_control_evidence_deep_lane.sh
bash scripts/compliance/test_generate_dsar_legal_hold_evidence_bundle.sh
bash scripts/compliance/test_run_dsar_legal_hold_contract_lane.sh
bash scripts/compliance/test_run_dsar_legal_hold_matrix.sh
bash scripts/compliance/test_run_dsar_legal_hold_deep_lane.sh
bash scripts/did/test_generate_federated_did_handshake_evidence_bundle.sh
bash scripts/did/test_run_federated_did_handshake_contract_lane.sh
bash scripts/did/test_run_federated_did_handshake_matrix.sh
bash scripts/did/test_run_federated_did_handshake_deep_lane.sh
bash scripts/task/test_generate_federated_delegation_settlement_evidence_bundle.sh
bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh
bash scripts/task/test_run_federated_delegation_settlement_matrix.sh
bash scripts/task/test_run_federated_delegation_settlement_deep_lane.sh
bash scripts/kolme/test_validate_version_compatibility.sh
bash scripts/kolme/test_run_version_compatibility_contract_lane.sh
bash scripts/governance/test_generate_governance_simulation_evidence_bundle.sh
bash scripts/governance/test_run_governance_simulation_contract_lane.sh
bash scripts/governance/test_run_governance_simulation_matrix.sh
bash scripts/governance/test_run_governance_simulation_deep_lane.sh
bash scripts/governance/test_generate_stake_slash_risk_evidence_bundle.sh
bash scripts/governance/test_run_stake_slash_risk_contract_lane.sh
bash scripts/governance/test_run_stake_slash_risk_matrix.sh
bash scripts/governance/test_run_stake_slash_risk_deep_lane.sh
bash scripts/reputation/test_generate_reputation_dispute_evidence_bundle.sh
bash scripts/reputation/test_run_reputation_dispute_contract_lane.sh
bash scripts/reputation/test_run_reputation_dispute_matrix.sh
bash scripts/reputation/test_run_reputation_dispute_deep_lane.sh
bash scripts/token/test_generate_token_launch_handoff_evidence_bundle.sh
bash scripts/token/test_run_token_launch_handoff_contract_lane.sh
bash scripts/token/test_run_token_launch_handoff_deep_lane.sh
bash scripts/treasury/test_generate_treasury_disbursement_evidence_bundle.sh
bash scripts/treasury/test_run_treasury_disbursement_contract_lane.sh
bash scripts/guard/test_run_durable_guard_recovery_contract_lane.sh
bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh
bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh
bash scripts/deploy/test_generate_staging_rehearsal_bundle.sh
bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh
cargo test -p kamn-core --test mainnet_cutover_runbook_docs
cargo test -p kamn-core --test release_gonogo_checklist_docs
cargo test -p kamn-core --test token_config_docs
cargo test -p kamn-core --test audit_export_interfaces_docs
cargo test -p kamn-core --test durable_guard_recovery_matrix
cargo test -p kamn-core --test durable_guard_snapshot_store
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
