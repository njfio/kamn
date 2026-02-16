# Issue #4693 Spec

- Title: `Task: decompose p2p transport and block pipeline monoliths with deterministic module contracts`
- Status: `Reviewed`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-block-pipeline/index.md`

## Problem Statement
`crates/kamn-core/src/p2p_transport.rs` and `crates/kamn-core/src/block_pipeline.rs` remain oversized and multi-responsibility, which increases maintenance burden and regression risk during transport/pipeline evolution.

## Scope
In:
- Extract cohesive transport/pipeline responsibilities into dedicated modules.
- Preserve deterministic fail-closed behavior and reason codes.
- Preserve runtime behavior via parity/regression tests.
- Document ownership boundaries and verification commands.

Out:
- Protocol redesign or wire-format changes.
- New transport providers or consensus model changes.

## Acceptance Criteria
- AC-1: Given `p2p_transport` extraction, when reviewing module ownership, then transport adapter/validation/lifecycle/error/runtime-event/swarm-stack/native-runtime responsibilities are delegated to dedicated submodules.
- AC-2: Given `block_pipeline` extraction, when reviewing module ownership, then validation, gossip ingress, fork-choice, evidence, and commit-store responsibilities are delegated to dedicated submodules.
- AC-3: Given existing transport/pipeline tests, when running scoped suites, then behavior remains parity-stable with deterministic fail-closed reason taxonomy.
- AC-4: Given docs updates, when reviewing runtime-network contracts, then transport/pipeline module ownership and verification commands are explicit.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration/Conformance | `rg -n "mod adapter|mod validation|mod lifecycle_regression|mod error|mod runtime_event|mod swarm_stack|mod native_runtime" crates/kamn-core/src/p2p_transport.rs` | p2p transport responsibilities delegated to dedicated modules |
| C-02 | AC-2 | Integration/Conformance | `rg -n "mod validation|mod gossip_ingress|mod fork_choice|mod evidence|mod commit_store" crates/kamn-core/src/block_pipeline.rs` | block pipeline responsibilities delegated to dedicated modules |
| C-03 | AC-3 | Functional/Regression | `cargo test -p kamn-core --test p2p_transport_runtime -- --nocapture` and `cargo test -p kamn-core --test block_pipeline -- --nocapture` | scoped transport/pipeline behavior remains stable |
| C-04 | AC-4 | Docs/Regression | docs review for transport/pipeline module ownership and command references | runtime-network docs include decomposition ownership map |

## Test Mapping
- C-01: `crates/kamn-core/src/p2p_transport/*.rs` extraction module wiring + runtime tests.
- C-02: `crates/kamn-core/src/block_pipeline/*.rs` extraction module wiring + pipeline tests.
- C-03: `cargo test -p kamn-core --test p2p_transport_runtime -- --nocapture`; `cargo test -p kamn-core --test block_pipeline -- --nocapture`; `cargo test -p kamn-core --test block_pipeline_sqlite_commit_store -- --nocapture`.
- C-04: `docs/foundation/runtime-network.md` update.

## Success Metrics
- `p2p_transport.rs` and `block_pipeline.rs` line surfaces are reduced with explicit ownership boundaries.
- Scoped transport/pipeline suites pass without behavior drift.
- Deterministic fail-closed reason taxonomy remains preserved.
