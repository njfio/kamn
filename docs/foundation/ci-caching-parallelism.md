# CI Caching and Parallelism Slice (Issues #182, #183)

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
- Added deterministic fast-lane target selection fallback safety:
  - Critical CI paths (`.github/workflows/*`, `scripts/ci/*`) escalate to full Rust validation scope.
  - Unknown non-doc paths escalate to full Rust validation scope.
  - duplicate/unknown matrix drift is guarded by selector regression tests (`Regression: #419`).
- Added narrow-diff telemetry summary metrics for CI budget artifact rollups:
  - narrow-diff records are defined as runs with `changed_files <= 3`.
  - summary reports narrow-diff elapsed and runner-minute means.
  - summary reports narrow-diff full-scope count to track safety fallback frequency (`Regression: #428`).

## Operational Guidance
- Cache invalidation:
  - Bump the shared key (`kamn-rust-ci-v1` -> `kamn-rust-ci-v2`) when cache corruption or systemic stale artifact behavior is observed.
  - Use a key bump only when necessary, because it triggers a full cache warm-up period.
- Cost control:
  - Keep cache saves on main only unless there is a measured need to populate PR-branch-specific caches.
  - Keep harness parallelism bounded (current limit: 2 in deep lane) to avoid unstable load spikes.
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
```
