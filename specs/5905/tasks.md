# Tasks: Issue #5905 - Persistent Service API WebSocket Streaming with Live Event Delivery

1. T1 (RED): add failing websocket integration/regression tests proving stream remains open and delivers follow-up events after message traffic.
2. T2 (GREEN): add runtime websocket event fan-out state to service API runtime.
3. T3 (GREEN): emit message lifecycle events from HTTP handlers into fan-out channel.
4. T4 (GREEN): replace one-shot websocket close logic with persistent stream forwarding loop.
5. T5 (VERIFY): run targeted websocket/service-api tests, fmt, strict clippy for `kamn-node`.
6. T6 (MUTATION): run diff-scoped mutation gate and close escaped mutants.
