# Issue #5315 Tasks

- [x] T1 (RED): add a failing test asserting `check_shell_rust_ratio_guardrail.sh` remains a thin delegator.
- [x] T2 (GREEN): move ratio-guardrail logic into `check_shell_rust_ratio_guardrail.py` and keep `.sh` as thin entrypoint.
- [x] T3 (VERIFY): run ratio-guardrail parity tests and #4000 performance fixture tests.
- [x] T4 (VERIFY): capture pre/post shell-ratio metrics and calculate combined #4000 + #5315 deltas.
- [ ] T5 (CLOSEOUT): update issue process log, open PR with shell-surface DoD markers, and merge.
