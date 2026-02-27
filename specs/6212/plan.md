# Plan: Issue 6212 - Cache Logging Config Instead of Per-Emission Env Reads

- Issue: #6212
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Introduce cached resolver helper in `logging.rs` backed by `OnceLock`.
2. Use cache-backed resolution path for non-test log emission.
3. Keep test-mode emission behavior stable to avoid cross-test env contamination.
4. Add targeted regression tests for cache hit and cached-failure behavior.
5. Run scoped verification for `kamn-node`.

## Affected Modules

- `crates/kamn-node/src/logging.rs`

## Risks and Mitigations

1. Risk: cache behavior could mask invalid config changes at runtime.
   - Mitigation: this is intended for startup-time config semantics; dynamic reload remains out of scope.
2. Risk: global cache can create brittle tests.
   - Mitigation: test cache logic via local `OnceLock` instances and keep test emission path uncached.
