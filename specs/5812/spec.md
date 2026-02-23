# Spec: Issue #5812 - Execute Live Harness S-02 Matrix Evidence Across SDK/CLI/MCP Modes

- Issue: #5812
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Issue `#5808` activated live `S-02` direct-message execution paths in all e2e harness drivers, but existing live execution evidence artifacts (`#5797`, `#5799`) only prove live runs for `S-01/S-04/S-06`. Without matrix evidence including `S-02`, live-service validation remains incomplete for the newly activated scenario path.

## Scope
In scope:
- Execute live harness matrix for `sdk-direct`, `cli-scripted`, and `mcp-tau` with scenarios `S-01,S-02,S-04,S-06`.
- Add/update deterministic research artifact markers proving per-mode and per-scenario outcomes, including `S-02`.
- Add docs-contract assertions (in existing docs-contract suite file) that fail closed on missing markers.
- Update milestone and lifecycle artifacts for issue closure.

Out of scope:
- Additional scenario activation beyond `S-02`.
- API/protocol/schema changes.
- Shell/workflow/template surface changes.

## Acceptance Criteria
- AC-1: Live harness executions complete for `sdk-direct`, `cli-scripted`, and `mcp-tau` using `S-01,S-02,S-04,S-06`, with captured JSON evidence outputs.
- AC-2: Research artifact contains deterministic marker set for mode statuses and scenario statuses including `S-02` status per mode.
- AC-3: Docs-contract test lane is RED before marker wiring and GREEN after marker wiring.
- AC-4: Milestone index and issue lifecycle artifacts reflect completed delivery.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | live run command for `sdk-direct` with `S-01,S-02,S-04,S-06` | command completes and output reports all four scenario results. |
| C-02 | AC-1 | Integration | live run command for `cli-scripted` with `S-01,S-02,S-04,S-06` | command completes and output reports all four scenario results. |
| C-03 | AC-1 | Integration | live run command for `mcp-tau` with `S-01,S-02,S-04,S-06` | command completes and output reports all four scenario results. |
| C-04 | AC-2 | Functional | `docs/research/e2e-live-testing-prd-r55-live-s02-execution-evidence.md` | contains required deterministic marker set including `S-02` coverage markers. |
| C-05 | AC-3 | Conformance | `cargo test -p kamn-e2e-harness --test docs_contract_release_group` | fails before markers are present, passes after wiring. |
| C-06 | AC-4 | Conformance | `specs/5812/{spec,plan,tasks}.md` + milestone index | lifecycle and milestone markers are updated to done state. |

## Test Mapping
- `cargo test -p kamn-e2e-harness --test docs_contract_release_group -- --nocapture` (RED -> GREEN)
- `cargo run -p kamn-e2e-harness -- run --mode sdk-direct ... --scenarios S-01,S-02,S-04,S-06`
- `cargo run -p kamn-e2e-harness -- run --mode cli-scripted ... --scenarios S-01,S-02,S-04,S-06`
- `cargo run -p kamn-e2e-harness -- run --mode mcp-tau ... --scenarios S-01,S-02,S-04,S-06`
- `cargo test -p kamn-e2e-harness -- --nocapture`

## Success Metrics / Observable Signals
- Live evidence confirms `S-02` pass outcomes across all three modes.
- Docs-contract suite enforces required marker presence for the new evidence artifact.
- Milestone reflects this slice as completed with no active issue drift.
