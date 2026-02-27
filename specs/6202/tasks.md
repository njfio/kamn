# Tasks: Issue 6202 - Runtime Commit Identity Must Use Payload Hash Value

- Issue: #6202
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): update regressions to assert equal-length distinct payload hashes produce different commit IDs.
- [x] T2 (GREEN): replace length-based payload component with value-based (hex-encoded) component.
- [x] T3 (REGRESSION): run `kamn-kolme` runtime identity policy test lanes.
- [x] T4 (VERIFY): run `cargo fmt --check`, scoped clippy, and scoped tests.
