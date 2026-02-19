# Issue #5192 Tasks

- Issue: #5192
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Ordered Tasks
- T1 (Tests/RED): introduce migration-inventory regression expectations for four legacy docs-contract files and add matrix inventory assertions in shared harness.
- T2 (Implementation/GREEN): migrate legacy marker assertions into `node_runtime_cli_docs` shared matrix and update docs command markers.
- T3 (Deletion/GREEN): remove superseded four test files once matrix parity is present.
- T4 (Verification): run targeted docs-contract suites and fix deterministic drift.
- T5 (Process): update issue process log, PR AC mapping, and shell-surface actual markers.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | shared harness case and marker inventory invariants |
| Functional | migrated document marker checks in shared matrix |
| Conformance | migration inventory removal assertions and stable case IDs |
| Regression | targeted docs lanes after file deletion and command-marker updates |
