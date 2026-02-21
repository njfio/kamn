# Milestone R50.22 - Service API Cross-Store Replay Telemetry Exposure

- Milestone: #120
- Primary issue: #5513

## Objective
Expose deterministic cross-store replay policy taxonomy metadata through `kamn-node` service API `/metrics` so runtime observability consumers can consume these markers directly.

## Scope
- Project cross-store replay reason taxonomy version and reason-code count into service API snapshot state.
- Emit additive metrics lines and enforce with service API endpoint tests.

## Out of Scope
- New dependencies.
- Non-additive wire/protocol changes.
