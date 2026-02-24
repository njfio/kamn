# Tasks: Issue #5903 - Replace Static Service API Observability with Live Runtime Telemetry

1. [x] T1 (RED): add failing service API endpoint tests proving `/metrics` and `/healthz` observability must reflect served traffic.
2. [x] T2 (GREEN): add runtime request telemetry tracker to service API runtime state.
3. [x] T3 (GREEN): wire telemetry recording into middleware success/error paths.
4. [x] T4 (GREEN): project runtime observability into live server `/metrics` and `/healthz` responses.
5. [x] T5 (VERIFY): run targeted service-api endpoint tests, fmt, strict clippy (kamn-node).
6. [x] T6 (MUTATION): run diff-scoped mutation gate for touched behavior and close escaped mutants.
