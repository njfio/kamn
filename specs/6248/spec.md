# Issue 6248 Spec

Status: Implemented
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
PR-triggered E2E validation was incomplete: CLI smoke ran on PR, but SDK-Direct and MCP lanes did not run in PR scope. That allowed non-CLI regressions to merge without pre-merge evidence.

## Scope
In scope:
- Update PR E2E workflow behavior so PRs execute CLI, SDK-Direct, and MCP smoke coverage.
- Add deterministic PR skip/fail reason markers for the required lanes.
- Update workflow contract tests/docs that guard PR E2E behavior.

Out of scope:
- Running the full scheduled E2E matrix on every PR.
- Introducing new non-smoke scenario families.

## Acceptance Criteria
- AC-1: PR workflow path executes three smoke lanes: CLI, SDK-Direct, and MCP Agent.
- AC-2: Any PR-lane skip/fail path is deterministic and emits explicit reason markers.
- AC-3: Workflow contract tests fail if PR coverage for any required lane is removed or silently downgraded.
- AC-4: Strategy/docs reflect the PR E2E lane matrix and skip marker contract.

## Conformance Cases
- C-01 (AC-1, Conformance): `.github/workflows/e2e-live.yml` runs SDK and MCP lanes in PR scope and keeps CLI PR smoke.
- C-02 (AC-2, Regression): workflow emits deterministic PR skip-reason markers for SDK, MCP, CLI lanes.
- C-03 (AC-3, Conformance): `crates/kamn-core/tests/e2e_live_workflow_lane.rs` fails when PR lane scope/smoke selectors/markers drift.
- C-04 (AC-4, Functional): `docs/ci/strategy.md` includes updated PR lane contract markers.

## Success Metrics
- PR merges include smoke coverage across CLI + SDK + MCP surfaces.
- Lane-scope drift is fail-closed by workflow contract tests.
