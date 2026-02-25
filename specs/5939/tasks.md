# Tasks: Issue #5939 - Task: Expand mutation and coverage gates (llvm-cov) for critical runtime/security paths

- Issue: #5939
- Spec: `specs/5939/spec.md`
- Plan: `specs/5939/plan.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): derive failing tests from all C-xx conformance cases before implementation.
- T2 (GREEN / Implementation): implement in-scope behavior changes with minimal diff.
- T3 (Refactor): improve structure/readability while preserving green tests.
- T4 (Regression): run targeted module tests plus issue-specific regression suites.
- T5 (Verify): run cargo fmt --check, strict clippy for touched crates, and scoped tests to close ACs.
- T6 (Process): update docs/spec status and attach AC evidence in PR + issue closure.

## Execution Result
- Added CI scripts/checkers:
  - `scripts/ci/check_critical_path_coverage.py`
  - `scripts/ci/run_critical_path_coverage_gate.sh`
  - `scripts/ci/run_critical_path_mutation_gate.sh`
  - `scripts/ci/test_check_critical_path_coverage.sh`
  - `scripts/ci/test_run_critical_path_mutation_gate.sh`
- Updated workflow gate wiring:
  - `.github/workflows/ci-fast-gate.yml`
- Updated evidence/documentation contracts:
  - `.github/pull_request_template.md`
  - `docs/ci/strategy.md`
  - `docs/security/secure-coding.md`
  - `docs/architecture/README.md`
  - `docs/architecture/adr-critical-path-assurance-gates.md`
  - `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
