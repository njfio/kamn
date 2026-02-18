# R27.44 Shell LOC deletion wave and hard ceiling governance

## Milestone Summary

Execution milestone for the next correction tranche after earlier shell-surface reduction work, focused on three explicit gaps:
- deletion wave of legacy scripts proven superseded by migration contracts,
- archival lifecycle policy for completed issue specs,
- hard shell LOC ceiling with downward-only shell-to-Rust ratio ratchet enforcement.

Primary objectives:
- drive shell LOC down by deleting superseded script families,
- reduce active spec maintenance surface via archive policy/tooling,
- fail closed in CI when shell budget or ratio trajectory regresses,
- institutionalize sustainable governance so shell LOC remains maintainable and below Rust LOC over time.

## Issue Hierarchy

- Epic: `#4954` — Epic: enforce legacy-script deletion, spec archival, and hard shell LOC ceiling sustainment
- Stories:
  - `#4955` — Story: execute explicit legacy-script deletion waves with supersession contracts
  - `#4956` — Story: implement spec archival lifecycle and completed-issue archive policy enforcement
  - `#4957` — Story: enforce hard shell LOC ceiling and downward-only shell-to-Rust ratio ratchet in CI
- Tasks:
  - `#4958` — Task: build superseded-script inventory and deterministic deletion-manifest contracts
  - `#4959` — Task: execute first deletion wave for superseded shell scripts with parity validation
  - `#4960` — Task: add stale-script reference detector and fail-closed CI guard for deleted entrypoints
  - `#4961` — Task: define spec archival policy and archive directory governance contracts
  - `#4962` — Task: implement spec archival tooling and active-vs-archived placement checker
  - `#4963` — Task: execute initial completed-spec archival wave and regression validation
  - `#4964` — Task: implement hard shell LOC ceiling policy checker with deterministic reason taxonomy
  - `#4965` — Task: wire shell ceiling and ratio-ratchet checks into CI fast gate as merge blockers
  - `#4966` — Task: enforce downward-only shell-budget ratchet updates and waiver governance workflow
- Subtasks:
  - `#4967` — Subtask: add red tests for superseded-script inventory schema and replacement evidence completeness
  - `#4968` — Subtask: implement inventory generator and deletion-manifest validator with reason-taxonomy outputs
  - `#4969` — Subtask: execute canary/ci/deploy/governance deletion wave with contract parity proof
  - `#4970` — Subtask: execute runtime/kolme deletion wave and remove orphan wrapper entrypoints
  - `#4971` — Subtask: add stale-reference detector tests for docs/workflows/manifests after script deletions
  - `#4972` — Subtask: wire stale-reference detector into ci-fast-gate and release go-no-go docs markers
  - `#4973` — Subtask: define archive layout and completed-spec retention policy markers
  - `#4974` — Subtask: implement specs archive tool and active-tree placement contract tests
  - `#4975` — Subtask: run first archive migration wave and publish archived-spec index report
  - `#4976` — Subtask: add hard shell LOC ceiling threshold fixture and checker red-green tests
  - `#4977` — Subtask: integrate ceiling+ratio ratchet checks into ci-fast-gate required status checks
  - `#4978` — Subtask: enforce ratchet-only threshold update workflow and waiver-mitigation issue linkage

## Governance Markers

- `shell_deletion_manifest_status=verified|fail-closed`
- `spec_archive_policy_status=verified|fail-closed`
- `shell_loc_hard_ceiling_status=within|exceeded`
- `shell_to_rust_ratio_ratchet_status=within|regressed`
- deterministic reason taxonomy versions and reason-code CSV fields on all policy outputs
