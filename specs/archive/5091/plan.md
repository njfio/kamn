# Issue #5091 Plan

- Issue: #5091
- Status: Implemented

## Approach
1. Replace M2 local agent DID checks with `AgentDid::parse` in:
   - DID session authentication requester validation.
   - Message-scope sender/recipient validation.
   - Agent-role requester validation path.
2. Preserve generic DID checks for non-agent role paths to avoid widening
   behavior changes in owner/auditor/platform role handling.
3. Add/adjust conformance tests for parser-specific malformed agent DID
   rejection and role-preserving behavior.
4. Run scoped and crate-level regression gates.

## Risks and Mitigations
- Risk level: high
- Risks:
  - Tightening parser checks can reject values that previously passed local
    format checks.
  - Over-tightening requester validation for non-agent roles can break existing
    owner/auditor deny-path expectations.
- Mitigations:
  - Apply `AgentDid` parsing only to explicit agent DID fields/paths.
  - Keep non-agent role behavior unchanged.
  - Lock behavior with deterministic tests.

## Interface Contract
- Additive/internal validation-path change.
- No dependency, protocol, or wire-format changes.

## ADR
- Not required for this scoped validation integration.
