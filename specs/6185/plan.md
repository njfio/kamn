# Plan: Issue 6185 - Service API TLS Default Hardening

- Issue: #6185
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Refactor TLS resolution to an env-injected helper:
   - mode env input,
   - cert env input,
   - key env input,
   - `allow_insecure_default` flag.
2. Keep runtime API (`resolve_service_api_tls_mode`) but route it through helper with:
   - `allow_insecure_default = cfg!(test)`.
3. For unset mode:
   - test mode => `Disabled`,
   - production mode => resolve required TLS cert/key or fail closed.
4. Add focused unit tests for production-simulated unset behavior and explicit mode compatibility.

## Affected Modules

- `crates/kamn-node/src/service_api_endpoint/server.rs`

## Risks and Mitigations

1. Regression in existing integration tests expecting HTTP defaults:
   - Mitigation: keep `cfg!(test)` insecure default path.
2. Behavior ambiguity for operators:
   - Mitigation: deterministic error messages for missing cert/key in production default path.

## Contracts / Interfaces

No public API shape change.
Security posture change: non-test default path is no longer implicit insecure HTTP.
