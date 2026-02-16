# Threat Control Matrix (Issues #142, #143)

This matrix translates PRD threat model concerns into enforceable controls, ownership, and deterministic validation tests.

## Control Matrix

| Threat ID | Threat | Control | Enforcement Point | Owner | Validation Test |
|-----------|--------|---------|-------------------|-------|-----------------|
| TM-001 | Forged instruction or sender spoofing | Require signed envelope verification against DID registry keys | SDK/client instruction verification pipeline | Security + Backend | `verify_instruction_signature_path` |
| TM-002 | Replay and nonce reuse | Enforce per-sender nonce monotonicity and stale state hash rejection | Transaction guard validation | Backend | `reject_out_of_sequence_nonce_per_sender` |
| TM-003 | Unauthorized escrow state mutation | Escrow lifecycle state machine blocks illegal transitions and invalid release amounts | Escrow lifecycle engine | Economics + Backend | `escrow_lifecycle_illegal_transition_rejected` |
| TM-004 | Invalid failover action during degraded quorum | Require listener/approver quorum checks before processor promotion | Failover runbook execution gate | DevOps + Governance | `failover_runbook_contains_failover_steps` |
| TM-005 | Signature metadata downgrade or algorithm drift | Enforce explicit signature algorithm/profile parsing and reject unsupported metadata pairs | Shared signer + transaction profile verification | Security + Backend | `integration_signature_profile_fixture_matrix_remains_consistent_with_transaction_guards` |
| TM-006 | Quorum attestation evidence drift or replayed approval artifact | Require deterministic quorum attestation schema checks and replay-guard policy validation before governance execution | Governance quorum attestation lane + policy checker | Governance + Security | `quorum_attestation_replay_guard_policy_contract` |
| TM-007 | Privileged role fallback bypass under secure-provider degradation | Deny local fallback for privileged signer roles and reject policy-blocked handshake downgrades | Signer policy contract lane + signer backend router | Security + Backend | `functional_privileged_roles_deny_fallback_when_provider_unavailable` |
| TM-008 | Validator/watchdog proof-consensus anomaly evidence missing or cadence/budget guard bypass | Require deterministic proof-consensus anomaly evidence with scheduled/manual deep-lane cadence and runtime budget policy checks | Runtime watchdog proof-consensus contract lane + deep lane policy checker | Runtime + Security | `run_watchdog_proof_consensus_contract_lane.sh` |
| TM-009 | Kolme live signature conformance drift or malformed parity evidence | Enforce secp256k1 signature parity vectors with deterministic NO-GO reason codes and policy gating | Kolme signature parity contract lane + local heavy validation matrix policy | Security + Crypto + QA | `test_run_signature_parity_contract_lane.sh` |
| TM-010 | Critical runtime output bypasses structured event contracts | Block ad-hoc `println!/eprintln!` usage in critical runtime and signer modules via deterministic source-contract tests | `kamn-node` runtime output contract tests | Backend + QA | `integration_runtime_output_contract_enforces_main_entrypoint_path` |

## Governance Quorum Attestation Replay Contract
- Fast lane:
  - `bash scripts/governance/run_quorum_attestation_replay_guard_lane.sh --output-file /tmp/governance-quorum-attestation-replay-report.json`
- Stable shell wrapper:
  - `scripts/governance/run_quorum_attestation_replay_guard_lane.sh`
- Shared Python implementation:
  - `scripts/governance/governance_quorum_attestation_replay_lane_contract.py`
- Policy checker:
  - `bash scripts/governance/check_quorum_attestation_replay_policy.sh --report-file /tmp/governance-quorum-attestation-replay-report.json`
- Stable shell wrapper:
  - `scripts/governance/check_quorum_attestation_replay_policy.sh`
- Shared Python implementation:
  - `scripts/governance/governance_quorum_attestation_replay_policy_contract.py`
- Contract lane:
  - `bash scripts/governance/run_quorum_attestation_replay_contract_lane.sh --output-file /tmp/governance-quorum-attestation-replay-contract-report.json`
- Required schema and reason-key markers:
  - `kamn.governance.quorum-attestation-replay-report.v1`
  - `governance_quorum_attestation_reason_codes:GO:v1`
  - `governance_quorum_attestation_reason_codes:NO-GO:v1`
- Required fail-closed policy:
  - quorum attestation evidence drift and replay attempts must fail closed (`Regression: #911`).

## Signer Privileged Fallback and Handshake Policy Contract
- Fast lane:
  - `bash scripts/signer/run_signer_policy_contract_lane.sh`
- Stable shell wrapper:
  - `scripts/signer/run_signer_policy_contract_lane.sh`
- Required signer policy checks:
  - `cargo test -p kamn-core --test signer_backend functional_privileged_roles_deny_fallback_when_provider_unavailable`
  - `cargo test -p kamn-core signer_backend::tests::router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes`
  - `cargo test -p kamn-core --test signer_backend regression_provider_client_backend_mismatch_is_rejected_without_fallback`
- Required fail-closed policy:
  - privileged-role fallback bypass attempts and policy-blocked handshake downgrades must fail closed (`Regression: #987`).

## Runtime Watchdog Proof-Consensus Policy Contract
- Fast lane:
  - `bash scripts/runtime/run_watchdog_proof_consensus_contract_lane.sh --output-file /tmp/watchdog-proof-consensus-contract.json`
- Scheduled/manual deep lane:
  - `KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE=scheduled bash scripts/runtime/run_watchdog_proof_consensus_deep_lane.sh --event-name schedule --output-json /tmp/watchdog-proof-consensus-deep-summary.json`
- Policy checker:
  - `bash scripts/runtime/check_watchdog_proof_consensus_policy.sh --bundle-file /tmp/watchdog-proof-consensus-contract.json`
- Required schema and reason-key markers:
  - `watchdog_proof_consensus_reason_codes:GO:v1`
  - `watchdog_proof_consensus_reason_codes:NO-GO:v1`
- Required fail-closed policy:
  - validator/watchdog proof-consensus anomaly evidence must fail closed for invalid/replay/mismatch outcomes.
  - cadence/budget guard bypass attempts must fail closed (`Regression: #996`).

## Kolme Signature Parity Conformance Contract
- Local contract lane:
  - `bash scripts/kolme/run_signature_parity_contract_lane.sh --output-json /tmp/kolme-signature-parity-matrix-report.json --policy-output-json /tmp/kolme-signature-parity-policy-report.json`
- Matrix runner:
  - `python3 scripts/kolme/run_signature_parity_matrix.py --fixture fixtures/kolme_commit/signature_parity_vectors.json --output-json /tmp/kolme-signature-parity-matrix-report.json`
- Policy checker:
  - `python3 scripts/kolme/check_signature_parity_policy.py --report-file /tmp/kolme-signature-parity-matrix-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-signature-parity-policy-report.json`
- Required reason-code markers:
  - `parity_signature_mismatch`
  - `parity_recovery_id_mismatch`
  - `parity_pubkey_mismatch`
- Required fail-closed policy:
  - known-bad signature vectors must emit deterministic NO-GO reason codes and fail closed on drift (`Regression: #2299`).

## Runtime Output Contract
- Contract test:
  - `cargo test -p kamn-node --test runtime_output_contract`
- Required fail-closed policy:
  - critical runtime/signer modules must not reintroduce ad-hoc `println!/eprintln!`
    output paths (`Regression: #4122`).

## Ownership and Review Cadence
- Security owner reviews this matrix each milestone and when new threat classes are introduced.
- Backend owner confirms code-level enforcement points remain linked to tests.
- Governance owner confirms policy/runbook controls remain operationally enforceable.

## Change Rules
- Do not add a control without naming a deterministic enforcement point.
- Do not add a threat row without at least one validation test identifier.
- Any modified row requires update to linked tests or explicit follow-up issue.
