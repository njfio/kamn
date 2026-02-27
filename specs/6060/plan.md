# Plan: Issue #6060

## Approach
1. Add RED tests in `crates/kamn-node/src/service_api_endpoint/auth.rs` to assert:
   - nonce floor survives guard re-initialization from same persistence path,
   - stale/equal nonces are rejected after restart,
   - higher nonce remains accepted.
2. Extend `ServiceApiReplayGuard` in `service_api_endpoint.rs` with:
   - per-sender nonce floor map,
   - optional persistence path,
   - load/persist helpers for replay state.
3. Initialize replay guard in `server.rs` from derived replay state path tied to service API state file.
4. Verify with targeted `kamn-node` tests and compile/lint gates.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `specs/6060/spec.md`
- `specs/6060/plan.md`
- `specs/6060/tasks.md`

## Risks / Mitigations
- Risk: Persistence write failures could allow replay gaps.
  Mitigation: fail-closed in replay-guard insert path when nonce floor cannot be persisted.
- Risk: Additional file I/O on auth path.
  Mitigation: persist only on accepted nonce updates, keep in-memory fast-path for duplicate checks.
- Risk: Behavior change for non-monotonic clients.
  Mitigation: explicit test coverage and deterministic rejection reason path via existing replay failure surface.

## Interfaces / Contracts
- Replay guard contract becomes monotonic per sender: accept only if `nonce > persisted_floor` and nonce is fresh in active replay window.
- Replay state persistence contract: JSON payload keyed by sender DID -> max nonce, loaded during runtime state initialization.
