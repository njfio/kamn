# Plan: Issue #6069

## Approach
1. Add RED tests in `server.rs` test module for missing TLS mode env resolution across loopback and non-loopback bind addresses.
2. Refactor TLS mode resolution to accept bind address context.
3. Implement bind-aware missing-env policy:
   - loopback => disabled
   - non-loopback => explicit configuration error.
4. Update call sites and run targeted verification.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `specs/6069/spec.md`
- `specs/6069/plan.md`
- `specs/6069/tasks.md`

## Risks / Mitigations
- Risk: startup behavior change for existing non-loopback deployments without TLS env.
  Mitigation: deterministic error message explains required env configuration.
- Risk: bind-address parsing edge cases.
  Mitigation: conservative parsing helper with explicit loopback detection for socket/localhost labels.

## Interfaces / Contracts
- TLS-mode resolution contract becomes bind-aware on missing env path.
