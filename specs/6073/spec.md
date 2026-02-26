# Spec: Issue #6073 - Service API sender DID to public-key binding

- Issue: #6073
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6072

## Problem Statement
Service API auth currently verifies signatures against one shared public key, while sender DID is self-declared. Any holder of that key can impersonate any sender DID.

## Scope
In scope:
- Add optional configuration for DID->public-key map.
- Use sender-specific key lookup for signature verification when map is configured.
- Reject auth when sender DID is not mapped in configured registry.
- Preserve existing single-key behavior when registry is absent.

Out of scope:
- DID registry protocol redesign.
- on-chain key ownership proofs.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: With DID key map configured, unknown sender DIDs fail auth.
- AC-2: With DID key map configured, signature verification uses sender-specific key.
- AC-3: Without DID key map configured, existing single-key auth mode remains unchanged.

## Conformance Cases
- C-01 (Unit, AC-1): key-selection helper returns no key for unknown sender when map configured.
- C-02 (Unit, AC-2): key-selection helper returns sender-specific key when sender exists.
- C-03 (Unit, AC-3): key-selection helper falls back to single key when map absent.

## Success Metrics / Observable Signals
- RED tests fail before DID-key selection helper exists.
- GREEN tests pass for configured-map and fallback modes.
- `cargo test -p kamn-node service_api_endpoint::auth::tests::` passes.
