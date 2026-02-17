# R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance

## Milestone Summary

Execution and sustainment milestone for `docs/plans/2026-02-17-shell-loc-reduction-plan.md` that maps phases 0-7 into active backlog and locks in future-process controls so shell LOC remains maintainable and below Rust LOC over time.

Current measured baselines (2026-02-17):
- Plan baseline: `115,524` shell LOC across `966` files + `84,455` Python LOC across `280` files.
- CI combined-surface baseline: `38,902` shell LOC, `125,087` Rust LOC, ratio `0.311`.

Milestone trajectory targets:
- Complete phase-mapped reductions (0-7) with deterministic conformance evidence.
- Reduce duplicated wrapper/helper/harness boilerplate and manual JSON construction.
- Sustain shell-to-Rust ratio below `1.0` via fail-closed CI and process-governance ratchets.

## Issue Hierarchy

- Epic: `#4859` — Epic: execute shell LOC reduction phases 0-7 and enforce shell-to-Rust sustainment
- Stories:
  - `#4860` — Story: deliver shared shell substrate and dispatch/wrapper reduction phases (0-2)
  - `#4861` — Story: consolidate wave/test/json shell boilerplate with reusable harnesses (phases 3-5)
  - `#4862` — Story: consolidate policy checkers and generate manifests from registry source (phases 6-7)
  - `#4863` — Story: institutionalize permanent CI and process contracts for shell-surface sustainment
- Tasks:
  - `#4864` — Task: migrate scripts to common.sh and remove duplicated helper boilerplate (phase 0)
  - `#4865` — Task: replace hardcoded dispatcher and wrapper mapping with manifest/registry resolution (phases 1-2)
  - `#4866` — Task: parameterize wave and matrix shell script families into definition-driven runners (phase 3)
  - `#4867` — Task: deploy shared shell test harness and JSON helper migration waves (phases 4-5)
  - `#4868` — Task: consolidate policy checkers into declarative framework (phase 6)
  - `#4869` — Task: generate manifests and wrappers from lane registry source of truth (phase 7)
  - `#4870` — Task: ratchet CI shell-surface budget and shell-to-Rust trajectory thresholds
  - `#4871` — Task: enforce shell-surface DoR/DoD contracts across AGENTS, templates, and PR docs
- Subtasks:
  - `#4872` — Subtask: implement common.sh primitives and migrate pilot script families
  - `#4873` — Subtask: execute bulk helper migration and compatibility verification wave
  - `#4874` — Subtask: add manifest wrapper_name/phase metadata and rewrite dispatcher resolution
  - `#4875` — Subtask: convert tiny wrappers to generated/symlink registry entrypoints
  - `#4876` — Subtask: consolidate framework wave wrapper matrix scripts into parameterized runner
  - `#4877` — Subtask: consolidate CI wave budget trend duplicate scripts into shared checker
  - `#4878` — Subtask: build scripts/lib/test_harness.sh and migrate first 75 high-duplication tests
  - `#4879` — Subtask: add JSON emit/write helpers and migrate top manual JSON-construction scripts
  - `#4880` — Subtask: implement declarative_policy_checker.py and schema conformance contracts
  - `#4881` — Subtask: migrate first 100 eligible policy checkers to declarative framework
  - `#4882` — Subtask: create lane registry source and manifest/symlink generation tooling
  - `#4883` — Subtask: retire manual manifest maintenance path and enforce registry drift contracts
  - `#4884` — Subtask: implement shell-rust telemetry collector with deterministic reason taxonomy outputs
  - `#4885` — Subtask: wire downward-only threshold ratchets into fast-gate policy checks
  - `#4886` — Subtask: update AGENTS and issue templates with mandatory shell-surface DoR/DoD fields
  - `#4887` — Subtask: add docs-contract and PR-template checks for shell governance markers

## Governance Markers

- Deterministic reason taxonomy outputs for shell-surface budget and ratio policy checks.
- Mandatory shell delta + mitigation fields in issue intake and PR verification.
- Downward-only threshold ratchets for script-surface trajectory controls.
