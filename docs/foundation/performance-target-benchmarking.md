# PRD 13.2 Performance Target Benchmark Evidence (Issue #184 / #595)

This document captures the first implementation slice for validating benchmark evidence against PRD Section 13.2 targets.

## Scope Delivered
- Added `crates/kamn-core/src/performance_targets.rs` with:
  - `PrdPerformanceTargets::v13_2()` target profile.
  - `PerformanceSample` benchmark input model.
  - `evaluate_performance_run(...)` deterministic aggregation + target evaluation.
  - `evaluate_performance_from_observability(...)` bridge from observability samples.
  - `PerformanceRunReport`, `PerformanceMetricResult`, and `PerformanceRunError` outputs.
  - `PerformanceRunReport::bottlenecks()` prioritized bottleneck list and remediation hints.
- Added integration tests in `crates/kamn-core/tests/performance_targets.rs`.

## PRD 13.2 Target Profile
| Metric | Target | Rule |
| --- | --- | --- |
| Message Latency (p50) | `< 100ms` | strict upper bound |
| Message Latency (p99) | `< 500ms` | strict upper bound |
| Throughput | `>= 10,000 msg/sec` | lower bound |
| Availability | `>= 99.9%` | lower bound |

## Deterministic Aggregation Rules
- `latency_p50_ms`: median across benchmark windows.
- `latency_p99_ms`: max across benchmark windows.
- `throughput_tps`: min across benchmark windows.
- `availability_pct`: min across benchmark windows.

These rules provide stable results for CI and avoid expensive full-load replay for every PR.

## Bottleneck and Remediation Output
Failed metrics are surfaced in this priority order:
1. Throughput
2. Latency p99
3. Availability
4. Latency p50

Each failed metric includes:
- observed value
- threshold value
- deviation percentage
- remediation hint (for example, processor backlog triage for p99 latency)

## Fast and Cost-Effective Validation Strategy
- PR gate (fast lane):
  - run targeted benchmark logic tests only (`performance_targets` + doc checks).
  - generate deterministic smoke report via `scripts/ci/generate_performance_smoke_report.sh --lane smoke`.
  - enforce thresholds with `scripts/ci/check_performance_thresholds.sh --lane smoke`.
  - no long-running synthetic load in PR jobs.
- Deferred deep validation (slow lane):
  - run deeper smoke/threshold checks (`--lane deep`) on schedule/nightly or manual dispatch.
  - keep heavy replay/load suites out of the PR critical path.
  - attach evidence bundles to issue/PR comments.

## Runtime Invariant/Fuzz/Concurrency Budget Contract (Issue #897)
- Combined bounded lane command:
  - `bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json`
- Policy checker command:
  - `bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json`
- Runtime budget environment variable:
  - `KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS=180` (default)
- Budget behavior:
  - lane fails closed when elapsed runtime exceeds the budget.
  - report schema remains deterministic: `kamn.runtime.invariant-fuzz-concurrency-contract-report.v1`.

## CI Threshold Gate Contract
- Threshold profile source: `.ci/performance-targets.env`.
- Required report metrics:
  - `latency_p50_ms` (must remain `< 100`).
  - `latency_p99_ms` (must remain `< 500`).
  - `throughput_tps` (must remain `>= 10000`).
  - `availability_pct` (must remain `>= 99.9`).
- Fast lane command sequence:
  - `bash scripts/ci/generate_performance_smoke_report.sh --lane smoke --output-json performance-smoke-report.json`
  - `bash scripts/ci/check_performance_thresholds.sh --lane smoke --report-json performance-smoke-report.json --profile-file .ci/performance-targets.env`
- Deep lane command sequence:
  - `bash scripts/ci/generate_performance_smoke_report.sh --lane deep --output-json performance-deep-report.json`
  - `bash scripts/ci/check_performance_thresholds.sh --lane deep --report-json performance-deep-report.json --profile-file .ci/performance-targets.env`

This keeps per-PR compute cost low while preserving confidence in target conformance trends.

## CI Runtime/Cost Measurement Signals
- `scripts/ci/summarize_budget_artifacts.sh` now emits narrow-diff telemetry slices from budget artifacts:
  - `Narrow-diff records (<=3 changed files)`
  - `Narrow-diff elapsed mean`
  - `Narrow-diff runner mean`
  - `Narrow-diff full-scope count`
- These signals provide lightweight evidence of fast-lane cost efficiency while preserving safety fallback visibility (`Regression: #428`).

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test performance_targets
cargo test -p kamn-core --test performance_target_benchmarking_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
