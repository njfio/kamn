# Tasks: Issue 6185 - Service API TLS Default Hardening

- Issue: #6185
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing unit tests for unset production-mode TLS resolution fail-closed behavior (`C-01`) and test-mode compatibility (`C-02`).
- [x] T2 (GREEN): implement env-injected TLS mode resolver with production fail-closed default and test-mode compatibility (`C-01`, `C-02`).
- [x] T3 (REGRESSION): verify explicit mode semantics (`disabled`, `require`) remain deterministic (`C-03`).
- [x] T4 (VERIFY): run `cargo fmt --check`, scoped clippy, and targeted `kamn-node` tests.

## Test Tier Mapping

- Unit: TLS mode resolver branch coverage (unset/default, explicit disabled, explicit require).
- Regression: existing service-api endpoint tests for TLS mode continue to pass.
