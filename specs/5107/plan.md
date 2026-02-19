# Issue #5107 Plan

- Issue: #5107
- Status: Implemented

## Approach
1. Introduce a canonical generic KAMN DID wrapper/parser in `did.rs` for non-agent DID roles while preserving existing `AgentDid` parser behavior.
2. Replace M2 local `validate_kamn_did` usage with canonical parser calls and remove duplicated local validation helper.
3. Enrich M2 DID errors with deterministic field-scoped reason taxonomy.
4. Add RED tests for requester/sender/recipient invalid DID field mappings, then implement and preserve existing M2 behavior contracts.
5. Run full regression + shell guardrails.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Introducing generic DID parser could alter accepted DID shapes.
  - Error-taxonomy changes could break existing tests expecting coarse errors.
- Mitigations:
  - Keep parser semantics aligned with existing `validate_kamn_did` contract.
  - Explicitly update and run conformance/regression tests for all impacted invalid DID assertions.

## Interface Contract
- Additive canonical DID type export in `did.rs`/`lib.rs`.
- M2 error taxonomy enriched but remains fail-closed.
- No protocol/wire changes.

## ADR
- Not required for this scoped integration task.
