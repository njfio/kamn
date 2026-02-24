# Spec: Issue #5909 - Fail Closed Insecure Deterministic Message Crypto by Default

- Issue: #5909
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`direct_message_crypto` and `group_channel_crypto` provide deterministic XOR-based fixture engines. In debug builds these engines are currently auto-enabled, so non-cryptographic paths remain reachable without explicit operator intent.

## Scope
In scope:
- Remove debug auto-enable behavior for insecure deterministic direct/group crypto engines.
- Require explicit environment opt-in for deterministic fixture crypto in all build profiles.
- Add regression tests that fail when opt-in markers are absent and pass when markers are present.
- Keep existing fixture behavior unchanged when explicit opt-in is set.

Out of scope:
- Replacing deterministic fixture crypto with production AEAD/key-agreement primitives.
- Protocol or wire-format redesign.

## Acceptance Criteria
### AC-1 Constructors fail closed by default
Given no insecure-crypto opt-in environment markers,
When constructing deterministic direct/group crypto engines,
Then constructors return deterministic `InsecureCryptoDisabled` errors.

### AC-2 Explicit local opt-in still works
Given insecure-crypto opt-in environment markers are explicitly set,
When constructing deterministic direct/group crypto engines,
Then constructors succeed and preserve existing fixture behavior.

### AC-3 Debug profile no longer bypasses policy
Given a debug/test runtime without explicit opt-in markers,
When deterministic direct/group crypto constructors are invoked,
Then they fail closed exactly as release profile behavior.

## Conformance Cases
- C-01 (AC-1, Unit): `DirectMessageCryptoEngine::new` rejects construction without opt-in env.
- C-02 (AC-1, Unit): `GroupChannelCryptoEngine::new` rejects construction without opt-in env.
- C-03 (AC-2, Unit): direct/group constructors succeed when explicit env opt-in is set.
- C-04 (AC-3, Regression): debug/test path does not auto-enable insecure deterministic crypto.

## Success Metrics / Observable Signals
- Deterministic fixture crypto is unreachable without explicit runtime opt-in.
- Existing fixture workflows remain available with explicit opt-in.
- New regression tests lock fail-closed policy and prevent debug auto-enable regressions.
