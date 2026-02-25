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
