# Tasks: Issue 6188 - Cryptographic DID-to-Key Binding

- Issue: #6188
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing tests for DID/key spoofing and restart continuity expectations.
- [x] T2 (GREEN): implement DID-to-key binding resolver/validation contract in auth path.
- [x] T3 (GREEN): persist and hydrate binding continuity state.
- [x] T4 (REGRESSION): run service-api auth, SDK client, and agent-lib auth-chain tests.
- [x] T5 (VERIFY): run `cargo fmt --check`, scoped clippy, and targeted cross-crate tests.
