# Spec: Issue #5797 — Execute Live Harness S-01/S-04/S-06 Across SDK/CLI/MCP

- Issue: #5797
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P1
- Status: Reviewed
- Last Updated: 2026-02-22

## Problem Statement
The harness has live probe implementations for S-01/S-04/S-06, but PRD progress requires actual live-service execution evidence across supported execution modes.

## Scope
- Execute live-enabled harness runs in modes `sdk-direct`, `cli-scripted`, and `mcp-any`.
- Use scenarios `S-01,S-04,S-06` for each mode.
- Collect deterministic run outputs and summarize outcomes/blockers in a research artifact.
- Finalize lifecycle/milestone metadata for this execution slice.

## Out of Scope
- Driver re-architecture.
- CI/workflow modifications.
- Runtime feature development beyond execution evidence.

## Acceptance Criteria

### AC-1: Multi-mode live execution attempted
Given the live harness command surface,
When each target mode is invoked with live toggles enabled,
Then all three mode runs execute and produce output artifacts.

### AC-2: Deterministic evidence captured
Given executed runs,
When outcomes are summarized,
Then a repository-tracked artifact records per-mode/per-scenario statuses and commands used.

### AC-3: Blockers are explicit and reproducible when failures occur
Given any failing scenarios,
When reporting is generated,
Then blocker reasons and reproduction commands are documented.

### AC-4: Lifecycle finalized
Given completed execution/reporting,
When closure occurs,
Then spec/tasks status and milestone slice metadata are updated.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | Run harness command for sdk-direct, cli-scripted, and mcp-any with live toggles. |
| C-02 | AC-2 | Integration | Evidence artifact contains commands + outcome matrix for S-01/S-04/S-06 by mode. |
| C-03 | AC-3 | Regression | Evidence lists blockers and reproducible command/environment prerequisites for failures. |
| C-04 | AC-4 | Functional | `specs/5797/spec.md`=Implemented, `specs/5797/tasks.md`=Completed, milestone updated. |

## Success Metrics / Observable Signals
- Executed run outputs available for all three modes.
- Evidence artifact clearly differentiates pass/fail and prerequisites.
- No ambiguity about next remediation steps for failed live paths.
