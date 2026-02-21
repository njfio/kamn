# Issue #5483 Plan - Topology Test Surface Decomposition

## Approach
1. Create subdirectory `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/`.
2. Move existing test blocks into cohesive subfiles (fingerprint/scope, topology-id mapping, coherence bundles, matrix/regression).
3. Replace root `live_postgres_topology_contract_tests.rs` with include hub statements.
4. Run targeted topology tests + fmt + strict clippy.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests/*.rs` (new)
- `specs/milestones/r50-7-daemon-topology-contract-test-decomposition/index.md`
- `specs/5483/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: include ordering/visibility regressions due split boundaries.
  - Mitigation: preserve original lexical order of tests and avoid changing function names or helper references.
- Risk: hidden coupling across test groups.
  - Mitigation: split only at top-level test boundaries and run targeted regression tests.

## Interfaces / Contracts
- No production API or protocol changes.
- Internal test-module contract: stable function names and deterministic include order.

## Validation Strategy
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_daemon_convergence_projection_fail_closed_reason_is_stable -- --exact`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features --manifest-path Cargo.toml -- -D warnings`
