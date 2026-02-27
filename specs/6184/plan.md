# Plan: Issue 6184 - Per-Agent Service API Authentication

- Issue: #6184
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Define per-agent signer key input and canonical verification contract.
2. Add DID->signer binding check in auth middleware path.
3. Remove/limit shared signer fallback behavior to non-production migration path only.
4. Add targeted auth mismatch and replay-protection regression tests.

## Affected Modules

- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store.rs` (binding persistence if needed)
- `crates/kamn-sdk/src/service.rs` / `crates/kamn-agent-lib/src/client.rs` (auth header propagation if required)

## Risks and Mitigations

1. Backward compatibility for existing SDK callers:
   - Mitigation: staged migration markers + explicit fail-closed error taxonomy.
2. Binding persistence complexity:
   - Mitigation: deterministic persistent store contract with narrow schema addition.
