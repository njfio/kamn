# Plan: Issue #6064

## Approach
1. Add RED tests in `crates/kamn-agent-lib/src/identity.rs` for production gating decision matrix and blocked-error behavior.
2. Introduce deterministic identity policy helper based on `debug_assertions` and opt-in env var.
3. Apply policy check in `AgentIdentity::from_agent_name` before key derivation.
4. Add explicit security warning doc comment for deterministic identity API.
5. Verify with targeted `kamn-agent-lib` tests, fmt, and clippy.

## Affected Modules
- `crates/kamn-agent-lib/src/identity.rs`
- `specs/6064/spec.md`
- `specs/6064/plan.md`
- `specs/6064/tasks.md`

## Risks / Mitigations
- Risk: behavior change in release builds for clients using deterministic identities.
  Mitigation: provide explicit env override and clear error message guiding migration to explicit key provisioning.
- Risk: global env reads in tests create flakiness.
  Mitigation: isolate decision logic in pure helper tests that accept injected env values.

## Interfaces / Contracts
- `AgentIdentity::from_agent_name` contract: deterministic derivation is non-production by default.
- Override contract: `KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY` enables deterministic derivation when production-mode policy would otherwise deny.
