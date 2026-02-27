# Plan: Issue 6188 - Cryptographic DID-to-Key Binding

- Issue: #6188
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Define binding source of truth (self-certifying DID suffix or persistent resolver map).
2. Add binding validation in service-api auth chain after cryptographic signature verification.
3. Add persistence/reload behavior for binding continuity across restart.
4. Add focused spoofing and restart regression tests.

## Affected Modules

- `crates/kamn-core/src/did.rs` (if DID parsing extension is required)
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs` / startup wiring
- SDK/agent-lib auth header plumbing if new signer material becomes mandatory

## Risks and Mitigations

1. Backward compatibility for existing DID forms:
   - Mitigation: explicit migration policy and deterministic errors.
2. Resolver persistence drift:
   - Mitigation: persisted state + startup hydration tests.
