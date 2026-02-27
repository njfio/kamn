# Issue 6230 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6223

## Problem Statement
E2E live validation currently runs post-merge (`push` on `main`), scheduled, or manual. Pull requests do not execute an E2E smoke lane, so regressions can merge before live-stack checks run.

## Scope
In scope:
- Enable pull-request triggering for `.github/workflows/e2e-live.yml`.
- Keep PR runtime bounded by routing PR runs to a smoke slice only.
- Add deterministic flaky-handling/rerun guardrails for PR smoke execution.
- Extend contract tests/docs to fail closed if PR E2E smoke wiring drifts.

Out of scope:
- Running the full 15-scenario matrix on pull requests.
- Adding new E2E harness modes.
- Replacing existing post-merge/scheduled E2E lanes.

## Acceptance Criteria
- AC-1: `e2e-live.yml` includes a `pull_request` trigger.
- AC-2: PR executions run a bounded E2E smoke lane (CLI mode, `S-01,S-02`) rather than full-matrix lanes.
- AC-3: PR smoke lane uses deterministic retry policy (`run_with_retry`, bounded attempts) for flaky handling.
- AC-4: Repository contract tests and strategy docs are updated to enforce and describe the PR E2E smoke wiring.

## Conformance Cases
- C-01 (AC-1, Conformance): Contract test fails if `pull_request` trigger is removed from `e2e-live.yml`.
- C-02 (AC-2, Conformance): Contract test fails if PR smoke job routing markers or smoke scenario selector (`S-01,S-02`) drift.
- C-03 (AC-3, Regression): Contract test fails if retry wrapper or bounded attempts markers are removed from PR smoke execution path.
- C-04 (AC-4, Functional): `docs/ci/strategy.md` contains deterministic E2E live workflow contract markers matching test reason-code taxonomy.

## Success Metrics
- PRs receive live E2E smoke signal before merge.
- Smoke lane remains bounded and deterministic.
- Workflow drift in PR E2E wiring is caught by existing CI contract tests.
