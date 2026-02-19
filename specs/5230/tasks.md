# Issue #5230 Tasks

- Issue: #5230
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): add deterministic invalid-DID contract tests for representative wave-C runtime/proof/reputation boundaries.
- T2 (Implementation/GREEN): migrate DID boundary validation + structured invalid-DID errors in `runtime_peer_coordination.rs` and `runtime_phase_coordination.rs`.
- T3 (Implementation/GREEN): migrate DID boundary validation + structured invalid-DID errors in `group_channel_crypto.rs` and `message_proof_anchoring.rs`.
- T4 (Implementation/GREEN): migrate DID boundary validation + structured invalid-DID errors in `reputation_signals.rs` and `reputation_state.rs`.
- T5 (Implementation/GREEN): migrate DID boundary validation + structured invalid-DID errors in `instruction_verify.rs`, `agent_upgrade_workflow.rs`, and `upgrade_orchestration.rs`.
- T6 (Regression): update impacted wave-C integration suites to keep valid-path behavior stable and assert deterministic invalid-DID markers.
- T7 (Verification): run targeted wave-C suites, `cargo fmt --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, and shell-ratio guardrail.
- T8 (Process): set `specs/5230/spec.md` to `Implemented`, update issue/PR AC mapping, and record shell-surface actual markers.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | module-level DID conversion helpers and field-level validation checks |
| Functional | runtime/proof/reputation wave-C boundary behavior |
| Integration | cross-module runtime lifecycle, proof anchoring, reputation, and upgrade flows |
| Regression | deterministic invalid-DID reason-code assertions across wave-C surfaces |
