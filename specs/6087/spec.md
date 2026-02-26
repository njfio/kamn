# Spec: Issue #6087 - Enforce production panic-surface checker in fast gate

- Issue: #6087
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`
- Last Updated: 2026-02-26
- Parent: #6084

## Problem Statement
The repository includes `scripts/ci/check_no_production_expect.py` for production-path panic-surface enforcement (`expect/panic/unreachable` and unsafe env fallback), but fast-gate currently enforces only `clippy::expect_used`. The broader policy must be executed directly in CI and produce deterministic report artifacts.

## Scope
In scope:
- Add fast-gate step to run `bash scripts/ci/check_no_production_expect.sh --output-json <report>`.
- Upload the generated panic-surface report as a CI artifact.
- Add/adjust workflow policy contract checks to fail closed if step/artifact wiring drifts.

Out of scope:
- Large changes to checker semantics and reason taxonomy.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Fast-gate executes production panic-surface checker under Rust scope and fails on violations.
- AC-2: Fast-gate uploads deterministic JSON report artifact for the checker output.
- AC-3: Workflow policy contract test enforces presence of checker command and artifact wiring.

## Conformance Cases
- C-01 (Conformance, AC-1): `.github/workflows/ci-fast-gate.yml` includes checker step gated by `steps.scope.outputs.run_rust == 'true'`.
- C-02 (Conformance, AC-2): `.github/workflows/ci-fast-gate.yml` uploads `ci-no-production-expect-report.json` artifact when present.
- C-03 (Regression, AC-3): `scripts/ci/test_workflow_scope_policy.sh` fails when checker command/artifact wiring is absent and passes when present.

## Success Metrics / Observable Signals
- Fast-gate enforces panic-surface policy in addition to clippy `expect_used`.
- CI artifacts include checker report JSON with stable schema/reason-taxonomy markers.
- Workflow drift that removes checker wiring fails policy contract tests.
