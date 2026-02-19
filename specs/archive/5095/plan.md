# Issue #5095 Plan

- Issue: #5095
- Status: Implemented

## Approach
1. Add internal typed contracts:
   - `DataLayerM2DidAuthRequestValidated`
   - `DataLayerM2MessageScopeValidated`
   with `AgentDid` fields for agent identities.
2. Implement `TryFrom` conversions from existing public string-based structs into
   typed validated contracts.
3. Refactor M2 auth and ABAC internals to use typed validated contracts after
   conversion at function boundaries.
4. Add RED tests for typed-conversion success/fail and ensure existing reason-marker
   behavior remains unchanged.
5. Run scoped/full regression gates and shell guardrails.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Refactor could change existing behavior in auth/ABAC paths.
  - Type conversion boundaries could introduce duplicate validation logic.
- Mitigations:
  - Keep public input structs unchanged; only add typed internal layer.
  - Validate once at boundary and reuse typed fields.
  - Lock behavior with current + new tests.

## Interface Contract
- Additive internal typed contracts.
- No dependency, protocol, or wire-format changes.

## ADR
- Not required for this scoped integration refactor.
