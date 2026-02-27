# Plan: Issue 6187 - Gate Deterministic Identity Derivation

- Issue: #6187
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Introduce deterministic identity gate helper in `identity.rs` using:
   - env input (`KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY`),
   - `allow_insecure_default` flag (`cfg!(test)` in production path).
2. Wire `AgentIdentity::from_agent_name` through the gate.
3. Add unit tests for:
   - production-simulated disabled default,
   - test-mode allowed default,
   - explicit non-test opt-in.
4. Run `kamn-agent-lib` unit/integration tests to ensure no regression.

## Affected Modules

- `crates/kamn-agent-lib/src/identity.rs`
- `crates/kamn-agent-lib/src/lib.rs` (doc updates only if needed)

## Risks and Mitigations

1. Breaking existing production callers using `connect()`:
   - Mitigation: deterministic, actionable error message and documented opt-in env.
2. Test regressions:
   - Mitigation: allow test-mode default in gate helper.

## Contracts / Interfaces

No API shape change.
Behavior change: `from_agent_name` is gated and disabled by default outside tests.
