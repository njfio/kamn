# Issue #5228 Tasks

- Issue: #5228
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): add/adjust wave-A tests to require structured invalid-DID reason markers and validated-boundary behavior.
- T2 (Implementation/GREEN): add validated DID wrapper conversions and deterministic error taxonomy updates in `bridge_adapter.rs`.
- T3 (Implementation/GREEN): apply validated DID wrapper conversions and deterministic error taxonomy updates in `cross_chain_bridge.rs`, `discord_bridge.rs`, `telegram_bridge.rs`.
- T4 (Implementation/GREEN): apply validated DID wrapper conversions and deterministic error taxonomy updates in `service_marketplace.rs`.
- T5 (Regression): update impacted integration tests (`bridge_outbound_quorum_execution`, `bridge_ingress_relay_harness`, `reputation_signal_routing`) to compile and preserve behavior.
- T6 (Verification): run targeted wave-A suites + `cargo fmt --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, and shell-ratio guardrail check.
- T7 (Process): set `specs/5228/spec.md` status to `Implemented`, update issue/PR with AC mapping and shell-surface actual markers.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | module-local validation helpers and conversion checks |
| Functional | bridge/marketplace boundary validation behavior |
| Integration | cross-module bridge and marketplace lane tests |
| Regression | deterministic reason-code assertions for invalid DID conversions |
