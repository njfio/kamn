# Tasks: Issue 6195 - Add Baseline Unit Coverage for Data Layer M0-M5

- Issue: #6195
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add baseline tests in M0 and M1 for append/hash-chain + merkle proof verification.
- [x] T2 (RED): add baseline tests in M2 and M3 for authz/search happy + fail-closed paths.
- [x] T3 (RED): add baseline tests in M4 and M5 for escrow/vector lifecycle happy + fail-closed paths.
- [x] T4 (GREEN): stabilize fixtures and adjust assertions until scoped `kamn-core` test lanes pass.
- [x] T5 (VERIFY): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, and scoped module tests.
