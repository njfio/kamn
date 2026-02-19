# CI Flaky Quarantine and Bounded Retry Slice (Issues #180, #181)

This document captures the first implementation slice for story #70.

## Scope Delivered
- Added quarantine-aware test command wrapper:
  - `scripts/ci/run_cargo_test_with_quarantine.sh`
  - Reads `.ci/flaky-tests.txt` and appends `--skip <test-id>` flags to `cargo test` commands.
- Integrated quarantine wrapper into CI scope selection:
  - `scripts/ci/select_targets.sh` now emits test commands that route through the quarantine wrapper for full, targeted, and smoke Rust test scopes.
- Added bounded retry policy regression checks:
  - `scripts/ci/test_workflow_retry_policy.sh` verifies retry limits remain bounded in fast and deep workflows.
- Added quarantine and target-selection regression checks:
  - `cargo test -p kamn-core --test shell_test_surface_migration_wave2`
  - `scripts/ci/test_select_targets.sh`

## Operational Guidance
- Quarantine entry lifecycle:
  - Every entry in `.ci/flaky-tests.txt` must include owner, test id, tracking issue, expiry, and notes.
  - Expired quarantine entries fail CI validation and must be removed or renewed with justification.
- Retry policy:
  - General test lanes are bounded to two attempts.
  - Deterministic invariant lanes are configured as no-retry (`--max-attempts 1`) to avoid hiding deterministic failures.
- Cost and stability:
  - Quarantine is intended to keep PR signal stable while active flakes are tracked and fixed.
  - Use quarantine as a short-lived control, not a permanent skip list.

## Local Validation
Run from repository root:

```bash
bash scripts/ci/test_ci_tools.sh
cargo test -p kamn-core --test shell_test_surface_migration_wave2
bash scripts/ci/run_cargo_test_with_quarantine.sh --dry-run -- cargo test -p kamn-core --test invariant_harness
```
