# Issue #5189 Tasks

- Issue: #5189
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Ordered Tasks
- T1 (Tests/RED): add Rust migration suite and ratio-policy suite with assertions that fail before CI/docs/command-surface wiring and deletions.
- T2 (Implementation/GREEN): wire `scripts/ci/test_ci_tools.sh` to run new Rust suites and remove migrated shell wrapper commands.
- T3 (Docs/Contracts/GREEN): update strategy docs and command-surface/strategy contract checks to require Rust migration lanes.
- T4 (Deletion): remove the 20 wave-1 shell wrapper files once Rust parity coverage is in place.
- T5 (Verification): run targeted fmt/tests/contracts and fix deterministic drift.
- T6 (Process): update issue/progress status, prepare PR with AC mapping + shell-surface delta markers.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | helper functions in ratio-policy parser/counting logic |
| Functional | Rust migration suite parity checks for all 20 removed wrappers |
| Conformance | migration inventory, command/docs markers, ratio schema/reason-code markers |
| Regression | fail-closed ratio threshold behavior and waiver gating |
