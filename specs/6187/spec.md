# Spec: Issue 6187 - Gate Deterministic Identity Derivation

- Issue: #6187
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P0
- Area: identity

## Problem Statement

`AgentIdentity::from_agent_name` deterministically derives private key material from a name
and is reachable from `KamnAgentHandle::connect`. This path is available in production by
default and is not explicitly gated as non-production behavior.

## Scope

In scope:
1. Add explicit runtime gate for deterministic identity derivation.
2. Keep test behavior deterministic for existing harnesses.
3. Return deterministic fail-closed error when gate is not enabled.

Out of scope:
1. DID-to-key cryptographic binding model redesign.
2. Auth architecture changes in `kamn-node`.
3. New key-management backends.

## Acceptance Criteria

### AC-1 Production Fail-Closed Default
Given deterministic identity gate is not enabled in production-mode resolution,
When `AgentIdentity::from_agent_name` is invoked,
Then it returns a deterministic error directing callers to explicit key provisioning.

### AC-2 Test Compatibility
Given test-mode resolution,
When `AgentIdentity::from_agent_name` is invoked,
Then deterministic derivation continues to work for local/integration tests.

### AC-3 Explicit Opt-In
Given opt-in environment marker is set in non-test mode,
When `AgentIdentity::from_agent_name` is invoked,
Then deterministic derivation is permitted.

## Conformance Cases

- C-01 (AC-1, Unit): production-simulated gate with missing opt-in returns deterministic failure.
- C-02 (AC-2, Unit): test-mode gate allows deterministic identity derivation.
- C-03 (AC-3, Unit): explicit opt-in allows deterministic identity derivation in production simulation.
- C-04 (AC-2/AC-3, Regression): existing agent-lib tests continue to pass.

## Success Signals

1. Deterministic key derivation is no longer silently available in production by default.
2. Existing tests remain stable without broad fixture churn.
3. Error text is actionable and deterministic.
