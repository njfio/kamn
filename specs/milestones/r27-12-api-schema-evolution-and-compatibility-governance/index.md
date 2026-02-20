# R27.12 API Schema Evolution and Compatibility Governance

## Milestone Summary

Closure tranche for deterministic API versioning and compatibility governance:
- supported-window version-policy enforcement,
- request/response compatibility classification,
- low-cost CI dry-run checker lanes with fail-closed taxonomy markers.

Milestone objective: keep API compatibility guarantees deterministic and auditable while preserving CI fast-gate budget boundaries.

## Issue Hierarchy

- Epic:
  - `#4038` — Epic: R27.12 close api schema evolution and compatibility governance gaps
- Stories:
  - `#4039` — Story: enforce deterministic api version policy and backward-compatible schema behavior
  - `#4040` — Story: validate compatibility matrix evidence with ci dry-run governance and marker parity
- Tasks (Story `#4039`):
  - `#4041` — Task: implement api version-policy checker with supported-window fail-closed enforcement
  - `#4042` — Task: implement request-response schema compatibility checker for supported version pairs
- Tasks (Story `#4040`):
  - `#4043` — Task: implement local-heavy api compatibility matrix lane with deterministic artifact schema
  - `#4044` — Task: add ci dry-run compatibility governance checker and docs-runbook parity contracts
- Subtasks (Task `#4041`):
  - `#4045` — Subtask: define version-policy fixture matrix and parser helper contracts
  - `#4046` — Subtask: add supported-window fail-closed checker and docs threshold parity checks
- Subtasks (Task `#4042`):
  - `#4047` — Subtask: implement schema comparator fixtures and compatibility classifier helper contracts
  - `#4048` — Subtask: add compatibility fail-closed regressions and taxonomy drift checks
- Subtasks (Task `#4043`):
  - `#4049` — Subtask: implement local-heavy compatibility matrix runner and deterministic artifact schema
  - `#4050` — Subtask: add compatibility matrix policy checker with fail-closed reason taxonomy
- Subtasks (Task `#4044`):
  - `#4051` — Subtask: implement ci dry-run compatibility checker and baseline threshold fixtures
  - `#4052` — Subtask: add docs-runbook parity contracts for compatibility thresholds and remediation markers

## Governance Markers

- `api_version_policy_supported_window=enforced`
- `api_compatibility_matrix=deterministic`
- `compatibility_ci_smoke_budget=fast_gate_safe`
