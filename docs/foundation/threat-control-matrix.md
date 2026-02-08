# Threat Control Matrix (Issues #142, #143)

This matrix translates PRD threat model concerns into enforceable controls, ownership, and deterministic validation tests.

## Control Matrix

| Threat ID | Threat | Control | Enforcement Point | Owner | Validation Test |
|-----------|--------|---------|-------------------|-------|-----------------|
| TM-001 | Forged instruction or sender spoofing | Require signed envelope verification against DID registry keys | SDK/client instruction verification pipeline | Security + Backend | `verify_instruction_signature_path` |
| TM-002 | Replay and nonce reuse | Enforce per-sender nonce monotonicity and stale state hash rejection | Transaction guard validation | Backend | `reject_out_of_sequence_nonce_per_sender` |
| TM-003 | Unauthorized escrow state mutation | Escrow lifecycle state machine blocks illegal transitions and invalid release amounts | Escrow lifecycle engine | Economics + Backend | `escrow_lifecycle_illegal_transition_rejected` |
| TM-004 | Invalid failover action during degraded quorum | Require listener/approver quorum checks before processor promotion | Failover runbook execution gate | DevOps + Governance | `failover_runbook_contains_failover_steps` |

## Ownership and Review Cadence
- Security owner reviews this matrix each milestone and when new threat classes are introduced.
- Backend owner confirms code-level enforcement points remain linked to tests.
- Governance owner confirms policy/runbook controls remain operationally enforceable.

## Change Rules
- Do not add a control without naming a deterministic enforcement point.
- Do not add a threat row without at least one validation test identifier.
- Any modified row requires update to linked tests or explicit follow-up issue.
