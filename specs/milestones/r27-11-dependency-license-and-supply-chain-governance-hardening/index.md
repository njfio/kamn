# R27.11 Dependency, License, and Supply-Chain Governance Hardening

## Milestone Summary
Closure tranche for deterministic dependency-risk policy checks, workspace license parity enforcement,
and SBOM/provenance governance evidence with bounded CI smoke costs and explicit local-heavy boundaries.

Milestone objective: keep dependency and supply-chain governance fail-closed, auditable, and fast-gate-safe.

## Issue Hierarchy

- Epic:
  - `#4023` — Epic: R27.11 close dependency risk, license parity, and supply-chain evidence gaps
- Stories:
  - `#4024` — Story: enforce deterministic dependency-risk governance with low-cost ci checks
  - `#4025` — Story: enforce license parity and sbom-provenance release evidence contracts
- Tasks (Story `#4024`):
  - `#4026` — Task: implement dependency-risk ci smoke checker with advisory threshold policies
  - `#4027` — Task: implement local-heavy deep dependency scan lane with policy-governed artifacts
- Tasks (Story `#4025`):
  - `#4028` — Task: enforce repository-crate license metadata parity with fail-closed checks
  - `#4029` — Task: generate deterministic sbom-provenance artifacts and release go-no-go parity checks
- Subtasks (Task `#4026`):
  - `#4030` — Subtask: implement advisory parser and dependency-threshold fixture contracts for ci smoke mode
  - `#4031` — Subtask: add ci smoke dependency checker and docs-contract threshold parity tests
- Subtasks (Task `#4027`):
  - `#4032` — Subtask: implement local-heavy deep dependency scan runner and deterministic artifact schema
  - `#4033` — Subtask: add deep-scan policy checker with ci dry-run governance and marker parity checks
- Subtasks (Task `#4028`):
  - `#4034` — Subtask: implement license parity checker across root license policy and crate manifests
  - `#4035` — Subtask: add regression and remediation-marker coverage for license metadata mismatches
- Subtasks (Task `#4029`):
  - `#4036` — Subtask: implement deterministic sbom-provenance generator and artifact schema validation
  - `#4037` — Subtask: add release go-no-go checker for sbom-provenance markers and docs parity contracts

## Governance Markers

- `dependency_ci_smoke_profile=bounded`
- `dependency_local_heavy_profile=opt_in`
- `workspace_license_policy=fail_closed`
- `sbom_provenance_release_gate=required`
