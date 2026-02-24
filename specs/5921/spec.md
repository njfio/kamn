# Spec: Issue #5921 - Task: Implement production DirectMessage/GroupChannel encryption (X25519 + XChaCha20-Poly1305)

- Issue: #5921
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
Current direct/group channel engines are deterministic XOR/FNV implementations and are not production-secure.

## Scope
In scope:
- Implement canonical key agreement + AEAD paths; mark deterministic fixtures test-only.

Out of scope:
- Protocol redesign outside canonical algorithm/profile contracts.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Production code paths use X25519 key agreement and XChaCha20-Poly1305 authenticated encryption only.
- AC-2: Deterministic test crypto is unreachable in production builds.
- AC-3: Interop, tamper, and negative-path tests validate confidentiality/integrity behavior.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Production code paths use X25519 key agreement and XChaCha20-Poly1305 authenticated encryption only.
- C-02 (Functional, AC-2): Verify Deterministic test crypto is unreachable in production builds.
- C-03 (Functional, AC-3): Verify Interop, tamper, and negative-path tests validate confidentiality/integrity behavior.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: crypto primitive wrappers and nonce/key handling
- Functional: direct/group message encrypt-decrypt flows
- Integration: cross-module message pipeline with real crypto
- Regression: deterministic fallback disabled in production
- Performance: bounded encrypt/decrypt overhead checks

## Dependencies
- #5916

