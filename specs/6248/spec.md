# Issue 6248 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
PR-triggered E2E validation remains incomplete: CLI smoke runs on PR, but SDK-Direct and MCP smoke lanes are skipped or non-PR-only. This allows regressions in non-CLI integration paths to merge without pre-merge evidence.

## Scope
In scope:
- Update PR E2E workflow behavior so PRs exercise CLI, SDK-Direct, and MCP smoke coverage (or explicit PR-safe equivalents).
- Add deterministic skip/fail reason codes for conditions where a lane cannot run.
- Update contract tests/docs that guard E2E workflow behavior.

Out of scope:
- Running the full scheduled E2E matrix on every PR.
- Introducing new non-smoke scenario families.

## Acceptance Criteria
- AC-1: PR workflow path executes three smoke lanes: CLI, SDK-Direct, and MCP Agent (or explicitly named PR-safe substitutes).
- AC-2: Any PR-lane skip is deterministic, fail-closed by policy, and emits explicit reason markers.
- AC-3: Workflow contract tests fail if PR coverage for any required lane is removed or silently downgraded.
- AC-4: Strategy/docs reflect the PR E2E lane matrix and skip policy.

## Conformance Cases
- C-01 (AC-1, Conformance): `.github/workflows/e2e-live.yml` PR path includes required smoke-lane jobs.
- C-02 (AC-2, Regression): Contract assertions detect missing/implicit skip rationale for PR lanes.
- C-03 (AC-3, Conformance): E2E workflow contract lane fails when required PR-lane markers are removed.
- C-04 (AC-4, Functional): `docs/ci/strategy.md` includes the updated PR E2E coverage contract.

## Success Metrics
- PR merges require smoke evidence from all required integration surfaces, not only CLI.
- E2E PR workflow behavior is deterministic and contract-tested.
