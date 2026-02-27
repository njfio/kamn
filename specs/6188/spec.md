# Spec: Issue 6188 - Cryptographic DID-to-Key Binding

- Issue: #6188
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P0
- Area: identity

## Problem Statement

Agent DID currently encodes only a syntactic name and has no intrinsic or resolver-backed
cryptographic relationship to signing key material.

## Scope

In scope:
1. Introduce deterministic DID-to-key binding contract in auth path.
2. Enforce mismatch rejection with deterministic reason taxonomy.
3. Persist DID/key binding state where required for restart durability.

Out of scope:
1. Full W3C DID document network resolver implementation.
2. Multi-signer key rotation policies.
3. Federation trust graph protocol design.

## Acceptance Criteria

### AC-1 Binding Contract Exists
Given sender DID and signer key,
When auth validation runs,
Then key acceptance requires a deterministic DID-to-key binding check.

### AC-2 Mismatch Fails Closed
Given binding conflict or spoofed sender DID/key pair,
When request is processed,
Then request is rejected with deterministic reason code.

### AC-3 Persistence / Restart Continuity
Given accepted DID-key binding material,
When node restarts,
Then binding continuity is retained for future auth checks.

## Conformance Cases

- C-01 (AC-1, Unit): valid DID-key binding passes cryptographic verification path.
- C-02 (AC-2, Unit/Integration): spoofed DID-key mismatch fails closed.
- C-03 (AC-3, Integration): binding state survives restart and enforces consistency.
