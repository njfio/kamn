# R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance

## Milestone Summary

Comprehensive script-surface reduction program to lower shell LOC, eliminate high-volume duplication, and enforce durable governance so shell LOC trends below Rust LOC over time.

Current measured baseline (2026-02-17):
- Shell LOC: `119,329` across `974` shell files (`175` symlink entries).
- Rust LOC: `54,601` (shell:Rust ratio `2.19`).
- Python LOC under scripts: `87,251`.

Milestone trajectory targets:
- Reduce shell LOC by phased deduplication and generator-driven architecture changes.
- Establish fail-closed CI/process contracts that prevent regression and force shell-surface accounting in future work.
- Reach and sustain shell LOC below Rust LOC (`<1.0` ratio).

## Issue Hierarchy

- Epic: `#4806` — Epic: reduce shell script surface and institutionalize shell-to-Rust LOC governance
- Stories:
  - `#4807` — Story: build shared shell substrate and eliminate dispatch/wrapper duplication
  - `#4808` — Story: collapse test/matrix/json shell boilerplate into reusable harnesses
  - `#4809` — Story: consolidate policy-checker/manifests into declarative generated architecture
  - `#4810` — Story: enforce permanent CI and process controls for shell-surface containment
- Tasks:
  - `#4811` — Task: introduce scripts/lib/common.sh and migrate duplicated shell boilerplate
  - `#4812` — Task: replace hardcoded dispatcher mapping and eliminate tiny exec wrappers
  - `#4813` — Task: consolidate wave and wrapper-matrix scripts into parameterized runners
  - `#4814` — Task: deploy shared test harness and JSON helper utilities across shell contracts
  - `#4815` — Task: introduce declarative policy-checker framework and migrate eligible contracts
  - `#4816` — Task: generate manifest and lane wiring artifacts from registry source of truth
  - `#4817` — Task: add fail-closed CI gates for shell-to-Rust ratio and script budget thresholds
  - `#4818` — Task: enforce shell-surface process contracts in issue templates, specs, and PR requirements
- Subtasks:
  - `#4819` — Subtask: implement scripts/lib/common.sh primitives and complete pilot migration wave
  - `#4820` — Subtask: execute bulk ROOT_DIR/usage/assert/extract helper migration with compatibility checks
  - `#4821` — Subtask: add wrapper_name/phase manifest fields and rewrite non-kolme dispatcher resolution
  - `#4822` — Subtask: implement wrapper exec registry dispatcher and replace <=8-line wrappers with symlinks
  - `#4823` — Subtask: replace framework wave wrapper matrix duplicates with wave-definition-driven runner
  - `#4824` — Subtask: replace CI wave budget trend duplicate scripts with parameterized checker
  - `#4825` — Subtask: introduce scripts/lib/test_harness.sh and migrate first 50 high-duplication tests
  - `#4826` — Subtask: add JSON emit/write helpers and migrate top 200 manual JSON construction scripts
  - `#4827` — Subtask: build declarative_policy_checker.py and declarative policy schema contracts
  - `#4828` — Subtask: migrate first 60 eligible *_contract.py checks and wrapper entrypoints
  - `#4829` — Subtask: create lane_registry source and manifest/symlink generation tooling
  - `#4830` — Subtask: retire static manifest maintenance path and add registry drift contract tests
  - `#4831` — Subtask: implement shell-rust LOC telemetry collector and fail-closed reason taxonomy outputs
  - `#4832` — Subtask: wire ratio and script-budget checks into CI fast gate with bounded runtime budgets
  - `#4833` — Subtask: update AGENTS/CONTRIBUTING and issue templates with shell-surface DoR/DoD gates
  - `#4834` — Subtask: add docs-contract and PR-template enforcement for script LOC delta and ratio trend markers

## Governance Markers

- Deterministic fail-closed reason taxonomies for shell-surface budget and ratio drift.
- Explicit CI smoke/local-heavy boundary controls for reduction work.
- Mandatory shell LOC delta and mitigation declarations in issue/PR process contracts.
