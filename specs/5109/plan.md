# Issue #5109 Plan

- Issue: #5109
- Status: Implemented

## Approach
1. Add M9 field-scoped invalid-DID reason constants.
2. Replace local `validate_kamn_did` with canonical `KamnDid`/`AgentDid` parser helpers and map parser errors into deterministic field-scoped M9 errors.
3. Update `DataLayerM9RealtimeDeliveryError::InvalidDid` to structured field/reason/detail form.
4. Add RED tests requiring new field-scoped taxonomy and then wire implementation.
5. Run full regression and shell guardrails.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Parser swap could alter edge-case owner scope behavior.
  - Error variant shape change can break existing tests.
- Mitigations:
  - Keep behavior additive and scoped to validation/error taxonomy.
  - Update only affected tests and run full `kamn-core` suite.

## Interface Contract
- Structured M9 invalid-DID error variant.
- Additive M9 reason constants exported via `lib.rs`.
- No protocol/wire/dependency changes.

## ADR
- Not required for this scoped integration task.
