# Issue #4693 Tasks

- Issue: `#4693`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red): add extraction-boundary contract tests for `p2p_transport` and `block_pipeline` root module declarations.
- T2 (Green): extract `p2p_transport` live/libp2p runtime responsibilities into focused submodule(s) and preserve API parity.
- T3 (Green): extract `block_pipeline` ingress/store/fork-choice responsibilities into focused submodule(s) and preserve API parity.
- T4 (Regression): validate deterministic transport/pipeline fail-closed reason-code selectors remain unchanged.
- T5 (Docs): update runtime-network ownership mapping + verification references.
- T6 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo test -p kamn-core --test transport_pipeline_module_extraction_contract`
  - `cargo test -p kamn-core p2p_transport::tests::transport_error_reason_code_remains_deterministic -- --exact`
  - `cargo test -p kamn-core p2p_transport::tests::coordinator_connect_and_advertise_transitions_to_active_state -- --exact`
  - `cargo test -p kamn-core block_pipeline::tests::block_pipeline_error_reason_code_extracts_commit_store_marker -- --exact`
  - `cargo test -p kamn-core block_pipeline::tests::regression_consensus_round_rejects_empty_mempool -- --exact`
  - `cargo test -p kamn-core --test runtime_network_docs`

## Completion Evidence
- Monolith roots reduced via module extraction with stable API.
- Deterministic fail-closed reason taxonomy preserved by regression selectors.
- Docs and extraction contracts verified.
