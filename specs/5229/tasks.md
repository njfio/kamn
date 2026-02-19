# Issue #5229 Tasks

- Issue: #5229
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): add wave-B deterministic invalid-DID regression tests for operator binding/actions, dashboard API/UI, governance workflow, and task payment workflows.
- T2 (Implementation/GREEN): migrate `operator_binding.rs` DID validation to typed wrappers with structured invalid-DID errors.
- T3 (Implementation/GREEN): migrate `operator_actions.rs` to typed binding error propagation with deterministic invalid-DID passthrough.
- T4 (Implementation/GREEN): migrate `operator_dashboard_api.rs` and `operator_dashboard_ui.rs` to validated DID boundaries + structured invalid-DID taxonomy.
- T5 (Implementation/GREEN): migrate `governance_workflow.rs` and `task_payment.rs` DID boundaries + structured invalid-DID taxonomy.
- T6 (Regression): update affected wave-B integration tests and module unit tests to assert reason-code contracts and preserve existing behavior.
- T7 (Verification): run targeted wave-B suites, `cargo fmt --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, and shell-ratio guardrail.
- T8 (Process): set `specs/5229/spec.md` to `Implemented`, update issue/PR AC mapping, and record shell-surface actual markers.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | DID conversion helpers and field-level validation checks in each wave-B module |
| Functional | operator/governance/task-payment boundaries with valid and invalid DIDs |
| Integration | operator dashboard, permissioned actions, governance workflow, and task payment suites |
| Regression | deterministic invalid-DID reason-code assertions across all wave-B modules |
