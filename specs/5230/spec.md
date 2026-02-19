# Issue #5230 Spec

- Title: Subtask: Typed-DID migration wave C for runtime, proof, and reputation surfaces
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
Wave-C runtime/proof/reputation modules still rely on raw DID strings at core boundaries. This allows malformed identity values to pass deeper into lifecycle, proof, and orchestration flows and yields inconsistent error surfaces.

## Scope
In:
- `crates/kamn-core/src/runtime_peer_coordination.rs`
- `crates/kamn-core/src/runtime_phase_coordination.rs`
- `crates/kamn-core/src/group_channel_crypto.rs`
- `crates/kamn-core/src/message_proof_anchoring.rs`
- `crates/kamn-core/src/reputation_signals.rs`
- `crates/kamn-core/src/reputation_state.rs`
- `crates/kamn-core/src/instruction_verify.rs`
- `crates/kamn-core/src/agent_upgrade_workflow.rs`
- `crates/kamn-core/src/upgrade_orchestration.rs`
- Targeted wave-C tests in `crates/kamn-core/tests/**`

Out:
- DID core internals (`did.rs`, `did_registry.rs`)
- Wave-A (`#5228`) and wave-B (`#5229`) modules
- Shell/python/workflow/template changes

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 260
- shell_to_rust_ratio_delta_estimate: -0.0013
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Wave-C module identity boundaries use typed DID conversions (`AgentDid` plus validated wrappers where non-agent/operator identities are required) before state/proof mutation.
- AC-2: Existing public API compatibility is preserved via validated conversion seams instead of breaking string-based payload contracts.
- AC-3: Invalid DID failures are deterministic and non-panicking with structured markers (`field`, `reason_code`, `detail`) and regression coverage.
- AC-4: Shell LOC remains unchanged.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Valid DID-bound runtime/proof/reputation inputs | Core logic consumes validated typed DID values before mutation |
| C-02 | AC-2 | Integration | Existing wave-C public call contracts | Existing call sites continue working through conversion wrappers |
| C-03 | AC-3 | Regression | Invalid DID at representative wave-C boundaries | Structured deterministic invalid-DID errors with reason markers |
| C-04 | AC-4 | Conformance | Diff-level shell/rust accounting + guardrail | shell delta stays zero |

## Test Mapping
- C-01/C-02/C-03:
  - `cargo test -p kamn-core --test runtime_peer_lifecycle`
  - `cargo test -p kamn-core --test message_proof_anchoring`
  - `cargo test -p kamn-core --test reputation_signal_routing`
  - `cargo test -p kamn-core --test reputation_state_model`
  - `cargo test -p kamn-core --test instruction_verification`
  - `cargo test -p kamn-core --test agent_upgrade_workflow`
  - `cargo test -p kamn-core --test upgrade_orchestration`
  - targeted module unit tests in `src/*`
- C-04:
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`

## Success Metrics
- Wave-C runtime/proof/reputation modules reject malformed DIDs before critical state transitions.
- Invalid DID errors become deterministic and machine-parseable in wave-C surfaces.
- Wave-C behavior contracts remain green with zero shell-surface growth.
