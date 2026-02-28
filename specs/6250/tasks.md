# Issue 6250 Tasks

- T1 (Red): Add Rust guardrail contract tests that initially fail against pre-migration fast-mode command surface expectations.
- T2 (Green): Update `scripts/ci/test_ci_tools.sh` to run the new Rust guardrail contract lane and retire the shell test wrapper invocation.
- T3 (Green): Remove `scripts/ci/test_check_shell_rust_ratio_guardrail.sh`.
- T4 (Green): Update `scripts/ci/test_ci_tools_command_surface_contract.sh` and `docs/ci/strategy.md` to reflect the migrated lane.
- T5 (Regression): Run targeted regression checks:
  - `cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json <file>`
- T6 (Verification): Record before/after ratio and shell/rust LOC evidence in `docs/planning/r59-followup.md`, map ACs to executed tests, and prepare PR closure evidence.
