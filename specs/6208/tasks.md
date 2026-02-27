# Tasks: Issue 6208 - Expose SDK Service Timeout Configuration

- Issue: #6208
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add timeout resolver tests for default, configured, and invalid env values.
- [x] T2 (GREEN): implement timeout resolver with env configuration and strict validation.
- [x] T3 (GREEN): wire socket read/write timeout setup through resolved timeout value.
- [x] T4 (REGRESSION): run `cargo test -p kamn-sdk`.
- [x] T5 (VERIFY): run `cargo fmt --check` and `cargo clippy -p kamn-sdk -- -D warnings`.

