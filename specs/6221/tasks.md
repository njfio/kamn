# Issue 6221 Tasks

- T1 (Red/Conformance): Add failing workflow contract assertion requiring explicit TLS mode markers for all three live lanes.
- T2 (Green/Implementation): Set `KAMN_SERVICE_API_TLS_MODE=disable` in each live lane run script before node startup.
- T3 (Regression): Run `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract`.
- T4 (Verification): Confirm AC-to-test mapping and update issue/PR evidence.
