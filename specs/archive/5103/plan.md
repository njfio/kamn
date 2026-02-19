# Issue #5103 Plan

- Issue: #5103
- Status: Implemented

## Approach
1. Add additive M10 projection request/report contracts for M8-derived partition shred completeness.
2. Implement bridge API on `DataLayerM10PartitionLifecycleRegistry` that:
   - validates owner scope + partition identity,
   - checks each supplied message ID via `DataLayerM8ComplianceRegistry`,
   - derives deterministic shred completeness and updates `all_messages_shredded`.
3. Add deterministic M10 error mapping for compliance lookup/projection failures.
4. Add RED tests for false/true completeness projection and fail-closed missing message behavior.
5. Run full regression + shell guardrails.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Bridge could accidentally change existing M10 archive/recoverability behavior.
  - Compliance error detail mapping could become unstable.
- Mitigations:
  - Keep existing M10 API unchanged; add additive projection only.
  - Use stable M10 reason codes for compliance projection failures.
  - Preserve existing M10 test corpus and run full crate regression.

## Interface Contract
- Additive M10 bridge types and method.
- No protocol/wire changes.
- No new dependencies.

## ADR
- Not required for this scoped integration task.
