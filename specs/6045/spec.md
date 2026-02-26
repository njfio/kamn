# Spec: Issue #6045 - Add production-target `expect()` contract gate

- Issue: #6045
- Status: Implemented
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5973

## Problem Statement
Audit discussions continue to conflate test-only `expect()` usage with production risk. The repository needs an explicit, enforceable gate that validates production targets only (`--lib --bins`) and fails closed when `expect()` is introduced in those targets.

## Scope
In scope:
- Add deterministic CI command contract for production-target `expect()` enforcement using `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::expect_used`.
- Wire the production-target gate into PR fast-gate workflow.
- Add contract/regression tests proving the command-surface includes `--lib --bins` and excludes test-only targets.
- Align CI strategy documentation with the enforced command.

Out of scope:
- Refactoring test-only `expect()` usage in unit/integration tests.
- Broad panic-path policy redesign outside production-target `expect()` enforcement.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: PR fast-gate executes a deterministic production-target `expect()` gate.
- AC-2: The gate command explicitly scopes to production targets (`--lib --bins`) and does not include test-only target flags.
- AC-3: Contract tests fail when command-surface or selector wiring drifts from the production-target scope.
- AC-4: CI strategy docs reflect the enforced production-target gate command.

## Conformance Cases
- C-01 (Conformance, AC-1): `ci-fast-gate` workflow includes a dedicated production-target `expect()` gate step.
- C-02 (Functional, AC-2): production-target checker executes `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::expect_used`.
- C-03 (Regression, AC-3): checker contract test fails when `--lib --bins` is removed or test-only targets are introduced.
- C-04 (Conformance, AC-4): CI strategy doc + doc-contract tests include the production-target gate command and scope marker.

## Success Metrics / Observable Signals
- `scripts/ci/test_check_no_production_expect_clippy.sh` passes.
- `scripts/ci/test_ci_tools.sh` fast-mode includes production-target expect checker contracts.
- `ci-fast-gate` workflow executes the gate under `run_rust == true`.
