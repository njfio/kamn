# Plan: Issue #6109

## Approach
1. Add RED unit tests in `kamn-core::did` for DID key-binding fingerprint generation and verification.
2. Implement `AgentDid` key-binding helper methods and deterministic fingerprint derivation from public key hex.
3. Add service-api auth enforcement helper for DID/public-key binding when DID-key map mode is active.
4. Introduce deterministic auth reason code for DID key-binding failures and update reason taxonomy constant.
5. Run targeted `kamn-core` and `kamn-node` tests, fmt, and clippy gates.

## Affected Modules
- `crates/kamn-core/src/did.rs`
- `crates/kamn-core/src/lib.rs` (exports if needed)
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `specs/6109/spec.md`
- `specs/6109/plan.md`
- `specs/6109/tasks.md`

## Risks / Mitigations
- Risk: breakage for existing plain DIDs.
  Mitigation: enforce binding only when DID-key-map auth mode is configured; fallback-mode behavior remains unchanged.
- Risk: reason taxonomy drift.
  Mitigation: add explicit reason constant and include in reason taxonomy CSV/tests.

## Interfaces / Contracts
- New `AgentDid` key-binding utility methods (non-breaking additive API).
- Service-api auth contract: DID-key-map mode requires sender DID key-binding validation against mapped public key.
