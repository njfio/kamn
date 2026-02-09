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
- Invariant-related changes (`invariants.rs`, `transaction.rs`, smoke/invariant harness tests, or harness scripts): run deterministic invariant harness in `fast` mode (single seed) after Rust tests.
- Runtime evaluator tests use direct unit-struct construction to avoid strict-clippy baseline noise (`Regression: #490`).

## Make and Selector Command-Surface Contract
Contributor entrypoint commands must remain stable and synchronized across `Makefile`, `README.md`, and selector routing policy:

- `make check`
- `make test`
- `make smoke-live-network`
- `make deep-live-network`
- `make demo`
- `make demo-localhost-transport`
- `make ci-tools`

Selector routing remains bounded through `scripts/ci/select_targets.sh`:

- `Makefile` changes map to runtime contract scope:
  - `run_runtime_snapshot_contract_tests=true`
  - `test_scope=runtime-contract`
- Localhost signed integration command changes map to dedicated scope:
  - `run_localhost_signed_integration_contract_lane_tests=true`
  - `test_scope=sdk-live-localhost-integration`
- Live transport parity command changes map to parity scope:
  - `run_live_transport_parity_contract_tests=true`
  - `live_transport_parity_languages=rust,python,typescript`
- Dashboard backend session/auth freshness command changes map to dashboard contract scope:
  - `run_dashboard_contract_tests=true`
  - `test_scope=frontend-contract`
- Frontend shell matrix command changes map to frontend scope:
  - `run_frontend_dashboard_tests=true`
  - `test_scope=frontend`
- Deployment SLO/rollback command changes map to deploy scope:
  - `run_deploy_preflight_tests=true`
  - `test_scope=deploy`

Required demo lane command contract:

- `bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed-integration-contract-report.json`
- `bash scripts/dashboard/run_backend_session_auth_freshness_contract_lane.sh --output-file /tmp/dashboard-backend-session-auth-freshness-contract-report.json`
- `bash scripts/frontend/run_dashboard_shell_determinism_matrix_contract_lane.sh --output-file /tmp/dashboard-shell-matrix-contract-report.json`
- `bash scripts/deploy/run_deployment_slo_rollback_contract_lane.sh --output-file /tmp/deployment-slo-rollback-contract-report.json`

Regression policy:

- make-target and selector workflow drift remains fail-closed (`Regression: #900`).
- command-surface parity drift remains fail-closed (`Regression: #939`).
- frontend shell matrix selector/docs parity remains fail-closed (`Regression: #943`).
- dashboard backend session/auth freshness selector/docs parity remains fail-closed (`Regression: #941`).
- deployment slo/rollback selector/docs parity remains fail-closed (`Regression: #944`).

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

## PR CI Impact Declaration
When CI-sensitive files are modified (`.github/workflows/*`, `scripts/ci/*`, `.ci/*`), PR description must explicitly declare CI impact.

Enforced by `scripts/ci/check_pr_ci_declaration.sh` in fast-gate.

## Script Regression Coverage
`ci-fast-gate` runs `scripts/ci/test_ci_tools.sh` to locally regression-test CI helper scripts:
- Budget evaluator (`test_evaluate_budget.sh`)
- Retry helper (`test_run_with_retry.sh`)
- Invariant harness runner (`test_run_invariant_harness.sh`)
- Selector matrix runner with output-env isolation (`test_select_targets.sh`, `Regression: #463`)
- Flaky registry validator (`test_check_flaky_registry.sh`)
- Budget summarizer (`test_summarize_budget_artifacts.sh`)
- PR CI declaration checker (`test_check_pr_ci_declaration.sh`)
- Flaky report commenter (`test_post_flaky_report_comment.sh`)
- Flaky issue syncer (`test_sync_flaky_registry_issues.sh`)

## Reporting and Burn-down
- Weekly workflow `ci-flaky-registry` validates the quarantine registry and publishes a report artifact.
- Weekly workflow `ci-flaky-report-comment` posts an automated report comment to issue `#70`.
- Weekly workflow `ci-flaky-sync-issues` labels and updates tracking issues referenced in `.ci/flaky-tests.txt`.
- Use `scripts/ci/summarize_budget_artifacts.sh` on downloaded `ci-budget-*.json` artifacts to compute p50/p95 and cache/retry trends.
- Use `scripts/ci/download_and_summarize_budget.sh --repo <owner/repo>` to pull recent budget artifacts and produce a local trend summary.

## Deep Validation Behavior
`ci-deep-validate` runs full formatting, linting, and test suites on a nightly schedule and manually on demand.
It also runs deterministic invariant harness coverage in `deep` mode (bounded seed set) to keep invariant negative-path checks off the PR-critical lane while preserving repeatable coverage.

## Cost Controls
- Concurrency cancellation enabled on both workflows.
- Rust dependency/build cache enabled in Rust lanes.
- Expensive suites are not on the PR merge-critical path.
- PR template includes a mandatory CI-impact declaration for workflow/test-scope changes.

## Post-Billing Runbook
- Follow `docs/ci/post-billing-closeout.md` to close #68/#70 once hosted workflows are available.
