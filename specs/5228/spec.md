# Issue #5228 Spec

- Title: Subtask: Typed-DID migration wave A for bridge and marketplace surfaces
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
Wave-A bridge and marketplace modules still accept raw DID `String` values at core boundaries. This preserves duplicate parsing paths and allows unchecked DID inputs to flow deeper into execution.

## Scope
In:
- `crates/kamn-core/src/bridge_adapter.rs`
- `crates/kamn-core/src/cross_chain_bridge.rs`
- `crates/kamn-core/src/discord_bridge.rs`
- `crates/kamn-core/src/telegram_bridge.rs`
- `crates/kamn-core/src/service_marketplace.rs`
- Targeted wave-A test updates in `crates/kamn-core/tests/**`

Out:
- Operator/governance wave-B modules (`#5229`)
- Runtime/proof/reputation wave-C modules (`#5230`)
- Shell/python/workflow/template changes

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 180
- shell_to_rust_ratio_delta_estimate: -0.0009
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Core wave-A execution paths use typed DID validated wrappers (`AgentDid`/`KamnDid`) instead of unchecked DID strings.
- AC-2: DID conversion failures return deterministic reason codes (field + reason marker) in module error taxonomies.
- AC-3: Existing bridge/marketplace behavior contracts remain green after migration.
- AC-4: Shell LOC remains unchanged.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Wave-A request/config inputs carrying valid DIDs | Inputs convert to typed wrappers and preserve existing behavior |
| C-02 | AC-2 | Regression | Invalid DID in each wave-A module boundary field | Error includes deterministic field + reason marker |
| C-03 | AC-3 | Integration | Bridge and marketplace contract test suites | Existing contract expectations remain green |
| C-04 | AC-4 | Conformance | Diff-level shell/rust accounting + guardrail check | shell delta remains zero |

## Test Mapping
- C-01/C-02/C-03:
  - `cargo test -p kamn-core --test bridge_adapter`
  - `cargo test -p kamn-core --test cross_chain_bridge`
  - `cargo test -p kamn-core --test discord_bridge`
  - `cargo test -p kamn-core --test telegram_bridge`
  - `cargo test -p kamn-core --test service_marketplace`
  - `cargo test -p kamn-core --test bridge_outbound_quorum_execution`
  - `cargo test -p kamn-core --test bridge_ingress_relay_harness`
  - `cargo test -p kamn-core --test reputation_signal_routing`
- C-04:
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`

## Success Metrics
- Wave-A module internals consume typed DID wrappers at core boundaries.
- Each module maps DID conversion errors to deterministic reason markers.
- Existing wave-A integration surfaces remain behaviorally stable and test-clean.
