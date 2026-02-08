# CI Caching and Parallelism Slice (Issues #182, #183, #568, #595)

This document captures the first implementation slice for CI runtime/cost optimization in story #68.

## Scope Delivered
- Enabled a shared Rust cache key across fast and deep CI workflows:
  - `shared-key: kamn-rust-ci-v1`
- Restricted cache writes to the main branch:
  - `save-if: github.ref == refs/heads/main`
  - PR runs restore caches but avoid repeated cache upload churn.
- Added bounded parallel execution support to deterministic invariant harness runs:
  - `scripts/ci/run_invariant_harness.sh --parallelism <n>`
  - Deep validation lane now uses `--parallelism 2`.
- Added CI regression checks for workflow cache policy and deep-lane parallel harness configuration.
- Added selector-driven CI-tool-check gating in fast-gate:
  - `run_ci_tool_checks` is true only for CI-sensitive diffs (`.github/workflows/*`, `scripts/ci/*`).
  - CI syntax, CI impact declaration, and CI tool regression steps are skipped for non-CI PR paths.
- Added deterministic fast-lane target selection fallback safety:
  - Critical CI paths (`.github/workflows/*`, `scripts/ci/*`) escalate to full Rust validation scope.
  - Deployment script paths (`scripts/deploy/*`) now route to a dedicated deploy preflight scope instead of full Rust.
  - Unknown non-doc paths escalate to full Rust validation scope.
  - duplicate/unknown matrix drift is guarded by selector regression tests (`Regression: #419`).
- Added narrow-diff telemetry summary metrics for CI budget artifact rollups:
  - narrow-diff records are defined as runs with `changed_files <= 3`.
  - summary reports narrow-diff elapsed and runner-minute means.
  - summary reports narrow-diff full-scope count to track safety fallback frequency (`Regression: #428`).
- Added deterministic PR/deep performance threshold gate commands:
  - `scripts/ci/generate_performance_smoke_report.sh --lane smoke|deep`
  - `scripts/ci/check_performance_thresholds.sh --lane smoke|deep --profile-file .ci/performance-targets.env`
  - PR fast-gate runs smoke lane only; deep-validate runs deep lane on schedule/manual to keep PR cost low.

## Operational Guidance
- Cache invalidation:
  - Bump the shared key (`kamn-rust-ci-v1` -> `kamn-rust-ci-v2`) when cache corruption or systemic stale artifact behavior is observed.
  - Use a key bump only when necessary, because it triggers a full cache warm-up period.
- Cost control:
  - Keep cache saves on main only unless there is a measured need to populate PR-branch-specific caches.
  - Keep harness parallelism bounded (current limit: 2 in deep lane) to avoid unstable load spikes.
  - Route deploy-only diffs to `scripts/deploy/test_preflight_topology.sh` so fast-gate avoids Rust toolchain startup.
  - Keep CI-tool-check steps scoped to CI-sensitive diffs to reduce unnecessary runner time (`Regression: #568`).
  - Keep performance threshold checks deterministic and fixture-based in PR lanes; reserve heavy replay/load suites for deferred deep lanes.
  - Prefer targeted crate scopes for known Rust module edits, but keep strict full-scope fallback for critical/unknown paths.
- Troubleshooting:
  - If deep invariant lane becomes unstable, temporarily reduce to `--parallelism 1` and compare budget telemetry.
  - If cache hit rates drop unexpectedly, verify lockfile churn and key continuity before changing workflow logic.
  - If narrow-diff full-scope count rises unexpectedly, inspect path classification rules before relaxing fallback policy.

## Local Validation
Run from repository root:

```bash
bash scripts/ci/test_ci_tools.sh
bash scripts/ci/run_invariant_harness.sh --mode deep --parallelism 2 --dry-run
bash scripts/ci/generate_performance_smoke_report.sh --lane smoke --output-json /tmp/perf-smoke.json
bash scripts/ci/check_performance_thresholds.sh --lane smoke --report-json /tmp/perf-smoke.json --profile-file .ci/performance-targets.env
bash scripts/deploy/test_preflight_topology.sh
```
