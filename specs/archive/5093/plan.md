# Issue #5093 Plan

- Issue: #5093
- Status: Implemented

## Approach
1. Add additive M9 control APIs:
   - channel-membership dispatch authorization against `ChannelStore`.
   - anti-spam-gated dispatch authorization against `AntiSpamEngine`.
   - combined controls dispatch path that composes membership + anti-spam +
     existing dispatch queue/backpressure semantics.
2. Add stable reason-code constants for channel-membership and anti-spam
   denials.
3. Extend M9 test suite with RED conformance cases for:
   - non-member denial,
   - anti-spam rejection mapping,
   - combined controls allow/deny behavior.
4. Keep existing M9 APIs and behavior intact for backward compatibility.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Integration could accidentally change existing M9 dispatch semantics.
  - Anti-spam rejection mapping could drift from stable reason taxonomy.
- Mitigations:
  - Implement additive APIs only; do not modify existing call paths.
  - Lock mapping with explicit tests for each rejection category.
  - Run scoped and crate-level regressions.

## Interface Contract
- Additive API only in M9 module.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped integration task.
