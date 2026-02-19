# Issue #5192 Tasks

- Issue: #5192
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Ordered Tasks
- T1 (Tests/RED): introduce migration-inventory regression expectations for four legacy docs-contract files and add matrix inventory assertions in shared harness.
- T2 (Implementation/GREEN): migrate legacy marker assertions into `node_runtime_cli_docs` shared matrix and update docs command markers.
- T3 (Deletion/GREEN): remove superseded four test files once matrix parity is present.
- T4 (Ratio Compliance/GREEN): add lightweight migration-contract shards to preserve shell-vs-rust test-file ratio non-regression while keeping assertions centralized.
- T5 (Verification): run targeted docs-contract suites, ratio-policy lane, and fix deterministic drift.
- T6 (Process): update issue process log, PR AC mapping, and shell-surface actual markers.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | shared harness case and marker inventory invariants |
| Functional | migrated document marker checks in shared matrix |
| Conformance | migration inventory removal assertions, migration-contract shard coverage, and stable case IDs |
| Regression | targeted docs lanes after file deletion/command updates plus shell-vs-rust ratio policy lane |
