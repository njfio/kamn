# R27.13 Authorization, Tenant Isolation, and Audit-Integrity Governance

## Milestone Summary

Closure tranche focused on deterministic authorization/isolation controls in production service paths:
- fail-closed authorization scope governance,
- tenant-isolation negative-matrix evidence,
- tamper-evident audit-integrity release checks.

Milestone objective: keep route-level authorization behavior deterministic and auditable without expanding shell governance surface.

## Issue Hierarchy

- Epic:
  - `#4053` — Epic: R27.13 close authorization, tenant-isolation, and audit-integrity governance gaps
- Stories:
  - `#4054` — Story: enforce deterministic authorization scope governance across protected service paths
  - `#4055` — Story: validate tenant-isolation evidence and audit-integrity release checks with fail-closed policy
- Tasks (Story `#4054`):
  - `#4056` — Task: implement authorization scope-policy checker with deterministic fail-closed taxonomy
  - `#4057` — Task: implement request-path authz matrix checks and docs parity governance
- Tasks (Story `#4055`):
  - `#4058` — Task: implement local-heavy tenant-isolation matrix lane with deterministic artifacts
  - `#4059` — Task: implement audit-evidence integrity checker and ci dry-run release-governance contracts
- Subtasks (Task `#4057`):
  - `#4062` — Subtask: implement route-level authz matrix fixtures and deterministic checker behavior
  - `#4063` — Subtask: add docs parity and remediation marker checks for authz route governance
- Subtasks (Task `#4058`):
  - `#4064` — Subtask: implement local-heavy tenant-isolation matrix runner and deterministic artifact schema
  - `#4065` — Subtask: add tenant-isolation policy checker with fail-closed leakage taxonomy and drift checks
- Subtasks (Task `#4059`):
  - `#4066` — Subtask: implement tamper-evident audit artifact generator and integrity hash verification helpers
  - `#4067` — Subtask: add ci dry-run audit-integrity checker and release go-no-go marker parity contracts

## Governance Markers

- `service_api_request_authz_scope_policy=fail_closed`
- `tenant_isolation_negative_matrix=required`
- `audit_integrity_release_gate=tamper_evident`
