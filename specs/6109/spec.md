# Spec: Issue #6109 - Cryptographic DID-to-key binding for service auth

- Issue: #6109
- Status: Reviewed
- Type: task
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6098

## Problem Statement
`AgentDid` currently represents only a syntactic identifier with no cryptographic key-binding claim. Service API auth can verify signatures, but the DID string itself does not cryptographically assert key ownership, leaving DID squatting and alias confusion risk when key-discovery context changes.

## Scope
In scope:
- Add deterministic public-key fingerprint binding primitives to `AgentDid` in `kamn-core`.
- Add DID/public-key binding verification in service-api auth when DID-key map mode is used.
- Emit explicit auth reason code on missing/invalid DID key binding.
- Add unit/regression tests covering binding creation, validation, mismatch, and auth failure behavior.

Out of scope:
- On-chain DID resolver deployment.
- Full DID document network resolution protocol.
- Backfilling historical DIDs in external deployments.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: `AgentDid` can express a deterministic key-binding fingerprint suffix derived from a public key.
- AC-2: `AgentDid` binding verification fails closed for missing binding, invalid key material, or fingerprint mismatch.
- AC-3: Service API auth enforces DID/public-key binding when DID-key-map mode is configured.
- AC-4: Auth failures from DID key-binding validation return deterministic reason code taxonomy entry.

## Conformance Cases
- C-01 (Unit, AC-1): bound DID generation embeds expected fingerprint marker and remains parseable.
- C-02 (Unit, AC-2): binding verification rejects missing binding and mismatched public key.
- C-03 (Unit, AC-2): binding verification accepts matching bound DID + public key.
- C-04 (Unit, AC-3): service-api auth binding check passes for mapped sender DID with matching key fingerprint.
- C-05 (Unit, AC-3/AC-4): service-api auth binding check fails with deterministic reason code when DID-key-map mode is enabled and binding is missing/mismatched.

## Success Metrics / Observable Signals
- New `AgentDid` key-binding tests pass in `kamn-core`.
- Service-api auth tests cover binding-pass and binding-fail flows.
- `cargo test -p kamn-core did::tests::` and `cargo test -p kamn-node auth::tests::` pass.
