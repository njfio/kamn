# PRD 13.2 Performance Target Benchmark Evidence (Issue #184)

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
  - enforce deterministic fixtures, no long-running synthetic load.
- Deferred deep validation (slow lane):
  - run heavier benchmark replay suites on schedule/nightly or manual dispatch.
  - attach evidence bundles to issue/PR comments.

This keeps per-PR compute cost low while preserving confidence in target conformance trends.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test performance_targets
cargo test -p kamn-core --test performance_target_benchmarking_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
