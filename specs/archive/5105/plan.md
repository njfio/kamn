# Issue #5105 Plan

- Issue: #5105
- Status: Implemented

## Approach
1. Add AgentDid-backed validation for M5 agent-scoped fields.
2. Extend M5 embedding record input/record with `ContentRetentionClass`.
3. Add owner-scoped retention-due projection API using `ContentLifecycleManager::retention_profile`.
4. Add deterministic M5 error taxonomy for invalid agent DID and retention projection input.
5. Add RED tests for new paths, then implement and run full regression + shell guardrails.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Input shape updates might break existing M5 tests unexpectedly.
  - Retention projection ordering could drift.
- Mitigations:
  - Update common test helper once and preserve existing behaviors.
  - Sort retention due output deterministically by due timestamp and embedding id.
  - Run full M5 + crate regression suite.

## Interface Contract
- Additive retention projection API.
- Input/record contracts updated to include retention class.
- No protocol/wire/dependency changes.

## ADR
- Not required for this scoped integration task.
