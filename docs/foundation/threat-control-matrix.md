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

## Governance Quorum Attestation Replay Contract
- Fast lane:
  - `bash scripts/governance/run_quorum_attestation_replay_guard_lane.sh --output-file /tmp/governance-quorum-attestation-replay-report.json`
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

## Ownership and Review Cadence
- Security owner reviews this matrix each milestone and when new threat classes are introduced.
- Backend owner confirms code-level enforcement points remain linked to tests.
- Governance owner confirms policy/runbook controls remain operationally enforceable.

## Change Rules
- Do not add a control without naming a deterministic enforcement point.
- Do not add a threat row without at least one validation test identifier.
- Any modified row requires update to linked tests or explicit follow-up issue.
