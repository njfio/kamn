# Tasks: Issue 6184 - Per-Agent Service API Authentication

- Issue: #6184
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing auth tests for shared-key impersonation and DID/key mismatch cases.
- [x] T2 (GREEN): implement per-agent signer binding validation and remove shared-key-only acceptance path.
- [x] T3 (REGRESSION): run service-api auth and SDK auth-chain integration tests.
- [x] T4 (VERIFY): run `cargo fmt --check`, scoped clippy, and targeted node/sdk test lanes.
