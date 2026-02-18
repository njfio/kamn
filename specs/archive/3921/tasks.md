# Issue #3921 Tasks

- Issue: #3921
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): define failing tests for runtime dispatch backpressure decision enforcement.
  - Evidence:
    - Implemented under child subtasks #3925 and #3926 with red-first test additions.
- [x] T2 (Green): wire deterministic queue action enforcement into live transport dispatch paths.
  - Evidence:
    - Delivered in PR #4998 (`fb44743a`).
- [x] T3 (Regression): add and run decision-matrix + dispatch-path reason-code drift tests.
  - Evidence:
    - Delivered in PR #4999 (`4a0ee603`).
    - `cargo test -p kamn-core runtime::tests::regression_runtime_backpressure_action_reason_matrix_remains_stable -- --exact --nocapture` -> pass.
    - `cargo test -p kamn-core --test p2p_live_transport_runtime functional_live_transport_dispatch_slow_producer_suspend_alias_stays_fail_closed -- --exact --nocapture` -> pass.
- [x] T4 (Docs): enforce runtime-network decision marker documentation parity.
  - Evidence:
    - `docs/foundation/runtime-network.md` updated and docs-contract test coverage landed in #4999.
- [x] T5 (Verify): validate bounded regression surfaces after downstream governance updates.
  - Evidence:
    - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture` -> pass.
    - `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/spec-archive-policy-post-wave2.json` -> `status=ok`.

## Completion Evidence
- Parent task scope is fully covered by merged subtasks #3925 and #3926 with deterministic tests and docs parity.
