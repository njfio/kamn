# Issue #4693 Spec

- Title: `Task: decompose p2p transport and block pipeline monoliths with deterministic module contracts`
- Status: `Reviewed`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4310`

## Problem Statement
`crates/kamn-core/src/p2p_transport.rs` and `crates/kamn-core/src/block_pipeline.rs` remain the largest files in the codebase and aggregate multiple responsibilities, raising review and regression risk.

## Scope
In:
- Extract clear module ownership boundaries for `p2p_transport` and `block_pipeline`.
- Preserve deterministic reason taxonomy and fail-closed behavior.
- Add extraction boundary regression checks.
- Update runtime network docs with new ownership mapping and verification references.

Out:
- Wire/protocol redesign.
- New transport providers or runtime modes.

## Acceptance Criteria
- AC-1: `p2p_transport.rs` delegates major live/libp2p runtime responsibilities to focused submodules while preserving existing public API.
- AC-2: `block_pipeline.rs` delegates ingress/store/fork-choice support responsibilities to focused submodules while preserving existing public API.
- AC-3: Deterministic fail-closed reason-code behavior for transport and block-pipeline paths remains unchanged.
- AC-4: Extraction boundaries are covered by regression/contract tests and documented in runtime-network docs.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | `cargo test -p kamn-core --test transport_pipeline_module_extraction_contract p2p_transport_module_boundaries_decompose_live_runtime_sections -- --exact` | p2p transport root declares focused submodules and no regression to monolithic ownership |
| C-02 | AC-2 | Regression | `cargo test -p kamn-core --test transport_pipeline_module_extraction_contract block_pipeline_module_boundaries_decompose_ingress_and_store_sections -- --exact` | block pipeline root declares focused submodules and no regression to monolithic ownership |
| C-03 | AC-3 | Unit | `cargo test -p kamn-core p2p_transport::tests::transport_error_reason_code_remains_deterministic -- --exact` | transport reason-code taxonomy remains stable |
| C-04 | AC-3 | Unit | `cargo test -p kamn-core block_pipeline::tests::block_pipeline_error_reason_code_extracts_commit_store_marker -- --exact` | block pipeline fail-closed reason extraction remains stable |
| C-05 | AC-3 | Functional/Integration | `cargo test -p kamn-core p2p_transport::tests::coordinator_connect_and_advertise_transitions_to_active_state -- --exact` | lifecycle/advertise behavior preserved |
| C-06 | AC-3 | Functional/Integration | `cargo test -p kamn-core block_pipeline::tests::regression_consensus_round_rejects_empty_mempool -- --exact` | pipeline fail-closed empty mempool behavior preserved |
| C-07 | AC-4 | Docs/Conformance | `cargo test -p kamn-core --test runtime_network_docs` | docs reflect ownership and verification contracts |

## Test Mapping
- `crates/kamn-core/tests/transport_pipeline_module_extraction_contract.rs`
- `crates/kamn-core/src/p2p_transport.rs`
- `crates/kamn-core/src/block_pipeline.rs`
- `docs/foundation/runtime-network.md`

## Success Metrics
- `p2p_transport.rs` and `block_pipeline.rs` line counts are reduced through extracted module ownership boundaries.
- Existing deterministic reason-code tests pass without behavior changes.
- Extraction-contract test(s) and runtime-network docs checks pass.
