# Issue 6230 Tasks

- T1 (Red): Extend `e2e_live_workflow_lane` contract assertions for PR smoke wiring and run test to capture failure before workflow updates.
- T2 (Green): Update `.github/workflows/e2e-live.yml` to add `pull_request` trigger and bounded PR smoke execution policy.
- T3 (Green): Add bounded retry wrapper markers to PR smoke execution path.
- T4 (Green): Update `docs/ci/strategy.md` E2E workflow contract marker block for the new reason taxonomy.
- T5 (Regression): Re-run `cargo test -p kamn-core --test e2e_live_workflow_lane` and verify pass.
- T6 (Verification): Map AC/C-cases in PR and close issue with deterministic evidence.
