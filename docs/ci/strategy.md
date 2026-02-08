# CI Strategy

## Goal
Keep CI feedback fast and runner cost low while preserving confidence.

## Lane Split
- `ci-fast-gate` (PR required): minimal critical path for merge decisions.
- `ci-deep-validate` (nightly/manual): heavier suites outside PR hot path.

## Stage-1 Budget Targets
- Fast gate runtime: <= 8 minutes p50, <= 12 minutes p95.
- PR runner consumption: <= 25 total runner-minutes.
- Nightly deep validate: <= 120 minutes.

Versioned thresholds are defined in `.ci/ci-budget.env`.

## Fast Gate Behavior
`ci-fast-gate` calls `scripts/ci/select_targets.sh` to select execution scope from changed files:

- Docs-only changes: run markdown hygiene check only.
- Rust changes in specific crates/manifests: run targeted clippy/tests by manifest path.
- Core Rust metadata changes (`Cargo.toml`, `Cargo.lock`, toolchain, `.cargo`): run full workspace lane.
- CI/workflow changes without Rust source changes: run shell syntax checks and a smoke Rust lane when a Cargo project exists.

## Budget Telemetry and Enforcement
Both lanes call `scripts/ci/evaluate_budget.sh` at the end of the run to:

- Compute elapsed runtime and approximate runner-minutes.
- Apply lane-specific warning/failure thresholds.
- Emit step-summary metrics for quick inspection.
- Upload JSON telemetry artifacts (`ci-budget-*.json`) for historical comparisons.

Policy:
- Warning at 90% of configured budget.
- Failure at 100% of configured budget for `ci-fast-gate` (merge-critical lane).

## Cache and Retry Telemetry
Telemetry includes:
- Rust cache hit status from `Swatinem/rust-cache` output.
- Whether bounded retry was used for test execution.

This data supports cache/parallel tuning and flaky-test burn-down without widening PR cost.

## Bounded Retry + Flaky Policy
- Tests run through `scripts/ci/run_with_retry.sh` with `max-attempts=2`.
- Retries are intentionally bounded to avoid hidden regressions.
- Flaky test quarantine inventory is tracked in `.ci/flaky-tests.txt`.
- Each quarantine entry must include owner, tracking issue, and expiry date.

## Deep Validation Behavior
`ci-deep-validate` runs full formatting, linting, and test suites on a nightly schedule and manually on demand.

## Cost Controls
- Concurrency cancellation enabled on both workflows.
- Rust dependency/build cache enabled in Rust lanes.
- Expensive suites are not on the PR merge-critical path.
- PR template includes a mandatory CI-impact declaration for workflow/test-scope changes.
