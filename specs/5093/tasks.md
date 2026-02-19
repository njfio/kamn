# Issue #5093 Tasks

- Issue: #5093
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing tests for channel-membership gating and anti-spam
  reason mapping in M9 dispatch controls.
- [x] T2 (Green): implement additive M9 control APIs integrating `ChannelStore`
  and `AntiSpamEngine`.
- [x] T3 (Refactor): centralize anti-spam rejection -> M9 reason mapping and
  keep existing M9 dispatch behavior unchanged.
- [x] T4 (Regression): run `cargo fmt --check`,
  `cargo clippy -p kamn-core -- -D warnings`,
  `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`, and
  `cargo test -p kamn-core`.
- [x] T5 (Governance): run shell guardrail evidence commands and confirm
  `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): mark spec/plan/tasks implemented/done and post issue closure
  evidence.
