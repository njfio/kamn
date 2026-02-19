# Issue #5215 Tasks

- Issue: #5215
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): add a root-line-budget decomposition contract test for `service_api_endpoint.rs` (fails before extraction).
- T2 (Implementation/GREEN): extract request-auth and payload helpers into submodules and wire root calls.
- T3 (Implementation/GREEN): extract routing and websocket helpers into submodules and wire root calls.
- T4 (Refactor): keep root as coordinator, remove duplicate code, and normalize module visibility/imports.
- T5 (Verification): run targeted service-api unit/functional/integration/regression suites and root-line-budget check.
- T6 (Process): update spec status to `Implemented`, update issue process log/status markers, and prepare PR evidence.
