# Spec: Issue #5797 - Execute Live Harness S-01/S-04/S-06 Across SDK/CLI/MCP Modes

- Issue: #5797
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Problem Statement
R54 confirms S-01/S-04/S-06 live probes exist across all harness drivers, but R55 requires execution evidence from true live runs. We need deterministic command evidence and per-mode scenario outcomes for `sdk-direct`, `cli-scripted`, and portable MCP mode.

## Acceptance Criteria
- AC-1: Live harness runs execute for `sdk-direct`, `cli-scripted`, and `mcp-tau` with live toggles enabled and `S-01,S-04,S-06` selected.
- AC-2: Evidence captures per-mode per-scenario outcomes (`PASS|FAIL|SKIP`) and overall mode status.
- AC-3: If any mode cannot complete, blockers are captured with exact repro commands, prerequisites, and observed error markers.
- AC-4: Lifecycle artifacts and milestone metadata are finalized for #5797.

## Scope
In scope:
- `specs/5797/spec.md`
- `specs/5797/plan.md`
- `specs/5797/tasks.md`
- `docs/research/e2e-live-testing-prd-r55-live-harness-5797-execution-evidence.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

Out of scope:
- New harness feature design beyond S-01/S-04/S-06 execution.
- CI workflow changes.
- Runtime architecture changes unrelated to live probe execution.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | Live run command for `sdk-direct` with `S-01,S-04,S-06` | Run completes and emits scenario results for all three scenarios. |
| C-02 | AC-1 | Integration | Live run command for `cli-scripted` with `S-01,S-04,S-06` | Run completes and emits scenario results for all three scenarios. |
| C-03 | AC-1 | Integration | Live run command for `mcp-tau` with `S-01,S-04,S-06` | Run completes and emits scenario results for all three scenarios. |
| C-04 | AC-2 | Functional | Parse result artifacts for each mode | Scenario status markers and overall mode status are documented deterministically. |
| C-05 | AC-3 | Regression | Failure path from any live run | Blocker section contains reproducible command, prerequisite gap, and failure marker. |
| C-06 | AC-4 | Conformance | Lifecycle docs + milestone metadata updates | #5797 appears as completed with artifacts status finalized. |

## Test Mapping
- `cargo run -p kamn-e2e-harness -- run --mode sdk-direct --scenarios S-01,S-04,S-06 ...`
- `cargo run -p kamn-e2e-harness -- run --mode cli-scripted --scenarios S-01,S-04,S-06 ...`
- `cargo run -p kamn-e2e-harness -- run --mode mcp-tau --scenarios S-01,S-04,S-06 ...`
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics
- Three live-mode harness executions attempted and recorded with deterministic outcomes.
- A single in-repo evidence artifact contains exact commands and outcome markers.
- Milestone metadata and lifecycle artifacts are marked complete and consistent.
