# Spec: Issue 6184 - Per-Agent Service API Authentication

- Issue: #6184
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P0
- Area: security

## Problem Statement

Service API auth currently validates request signatures against one shared configured public key,
while sender DID is client-declared. This creates systemic impersonation risk for all agents
sharing that key.

## Scope

In scope:
1. Remove single-shared-key auth dependency from protected request validation path.
2. Introduce per-agent signer key binding at auth time.
3. Preserve deterministic reason-code failures for missing/invalid binding.

Out of scope:
1. External DID resolver network integration.
2. Key rotation governance policies.
3. Cross-node federated identity synchronization.

## Acceptance Criteria

### AC-1 No Shared-Key-Only Auth
Given protected routes,
When auth validation runs,
Then signature verification is not anchored to one global shared signer key.

### AC-2 Per-Agent Binding Enforcement
Given sender DID and signer key material,
When request is verified,
Then sender DID must match a deterministic signer binding contract and mismatch fails closed.

### AC-3 Regression Safety
Given existing protected route auth tests,
When per-agent binding is introduced,
Then deterministic reason-code contracts remain green or are updated explicitly with migration markers.

## Conformance Cases

- C-01 (AC-1, Unit): shared-key-only auth path is no longer sufficient for protected request acceptance.
- C-02 (AC-2, Unit/Integration): sender DID/signer key mismatch fails with deterministic reason code.
- C-03 (AC-3, Regression): existing auth-chain tests remain green under new binding model.
