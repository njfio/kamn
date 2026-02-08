# Multi-AZ Topology and Processor Failover Runbook (Issues #140, #141)

This runbook defines the minimal multi-AZ deployment shape and deterministic failover process for Processor continuity.

## Multi-AZ Topology

### AZ Layout
- `AZ-a`: primary Processor, Listener-1
- `AZ-b`: standby Processor, Approver-1
- `AZ-c`: Listener-2, Approver-2

### Baseline Assumptions
- At least one standby Processor is healthy and synchronized.
- Listener and Approver quorum remains available during failover.
- Persistent storage and state snapshots are replicated across AZ boundaries.
- Gossip network connectivity is available between all validator roles.

## Deployment Preflight
Run topology validation before rollout:

```bash
bash scripts/deploy/preflight_topology.sh \
  --processors 3 \
  --listeners 3 \
  --approvers 3 \
  --required-approvals 2
```

Preflight rejects invalid cardinality and quorum mismatches (Regression: #481).

## Processor Failover Procedure

1. Detect processor failure
- Trigger on missing block production heartbeat and failed health checks.
- Confirm failure persists beyond configured grace threshold.

2. Validate listener and approver quorum
- Verify Listener set can still attest inbound events.
- Verify Approver quorum is intact for outbound authorization gating.

3. Promote standby processor
- Freeze old Processor identity in deployment metadata.
- Promote standby Processor instance in AZ-b.
- Ensure promoted node starts with current state schema/version and last committed state hash.

4. Verify chain continuity
- Confirm no gap in block height sequence after promotion.
- Confirm mempool replay does not duplicate previously committed transactions.
- Confirm block producer role ownership switched to promoted node.

5. Resume normal operations
- Re-enable autoscaling policies paused during failover.
- Start recovery workflow for failed AZ component.

## Verification Checklist
- [ ] State hash continuity confirmed
- [ ] No duplicate block production
- [ ] Quorum health green for Listener and Approver sets
- [ ] Processor metrics stable (block cadence, mempool depth, commit latency)
- [ ] Incident timeline recorded with promotion timestamp and operator identity

## Rollback Guidance
- If promoted Processor fails verification, pause block production and restore previous stable snapshot.
- Re-run quorum checks before attempting a second promotion.
- Escalate to incident commander if two consecutive promotion attempts fail.
