# ADR: Critical-Path Mutation and Coverage Gates

- Status: Accepted
- Date: 2026-02-25
- Deciders: Runtime/Security/QA maintainers
- Issue: #5939

## Context

Pre-merge CI had strong unit/integration execution but no deterministic,
automated mutation + coverage guardrail scoped to the highest-risk runtime and
security modules. This left escaped-mutant remediation and critical-path
coverage assertions too manual.

## Decision

1. Add bounded critical-path coverage gating in `ci-fast-gate` workspace
   pre-merge job via:
   - `scripts/ci/run_critical_path_coverage_gate.sh`
   - `scripts/ci/check_critical_path_coverage.py`
   - `.ci/critical-path-coverage-thresholds.json`
2. Add bounded critical-path mutation gating in `ci-fast-gate` workspace
   pre-merge job via:
   - `scripts/ci/run_critical_path_mutation_gate.sh`
   - deterministic slice selectors across:
     - `kamn-core` direct/group crypto and HTTP transport
     - `kamn-node` runtime orchestration, service API endpoint, signer
3. Upload coverage and mutation reports as CI artifacts for every PR run.

## Consequences

- Positive:
  - PRs now carry concrete, machine-checked mutation and coverage evidence for
    critical runtime/security code paths.
  - Escaped mutants and coverage regressions fail closed with deterministic
    reason taxonomies.
  - Assurance evidence becomes auditable as artifacts instead of ad-hoc logs.
- Trade-offs:
  - Workspace pre-merge runtime increases due `cargo-llvm-cov` and
    `cargo-mutants` installation/execution.
  - Thresholds are intentionally scoped and conservative; they must be ratcheted
    upward as coverage depth expands.

## R59 Threshold Ratchet Baseline

Issue `#6229` raised coverage minima using measured deterministic probe output from
`scripts/ci/run_critical_path_coverage_gate.sh` and preserved fail-closed policy
semantics.

| Target | Prior Min (line/function) | Measured Baseline (line/function) | New Min (line/function) | Rationale |
|---|---:|---:|---:|---|
| `crates/kamn-core/src/direct_message_crypto.rs` | `50.0 / 50.0` | `73.73 / 65.85` | `60.0 / 60.0` | Added deterministic direct-message roundtrip and legacy compatibility tests, then ratcheted to enforce broad path coverage with headroom. |
| `crates/kamn-core/src/group_channel_crypto.rs` | `24.0 / 30.0` | `58.63 / 56.60` | `50.0 / 50.0` | Previous thresholds were placeholder-level; new minima require meaningful execution depth while retaining margin. |
| `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs` | `25.0 / 20.0` | `28.45 / 22.95` | `27.0 / 22.0` | Raised to near current deterministic floor for this bounded selector set without introducing flakiness. |
| `crates/kamn-node/src/runtime_orchestration.rs` | `15.0 / 18.0` | `16.00 / 18.42` | `15.0 / 18.0` | Held constant pending broader runtime selector expansion; current baseline has minimal stable headroom. |
| `crates/kamn-node/src/service_api_endpoint.rs` | `40.0 / 20.0` | `47.80 / 25.64` | `45.0 / 24.0` | Increased to enforce stronger contract/behavior path execution while preserving deterministic margin. |
| `crates/kamn-node/src/signer.rs` | `2.0 / 2.5` | `5.36 / 10.26` | `4.5 / 8.5` | Added deterministic signer retry/backoff and key-parse zeroization probes, then raised from ineffective placeholder values. |

The coverage gate remains deterministic and fail-closed because only explicit
`--exact` test selectors and static threshold files are used; no policy schema or
reason taxonomy behavior was changed.
