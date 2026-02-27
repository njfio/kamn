# Tasks: Issue 6190 - SDK WebSocket Extended-Length Frame Support

- Issue: #6190
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing unit tests for 16-bit and 64-bit extended frame length parsing and truncation guards (`C-01`, `C-02`, `C-03`).
- [x] T2 (GREEN): implement extended-length unmasked text frame parser and wire into `read_event_once` (`C-01`, `C-02`).
- [x] T3 (REGRESSION): run existing websocket route integration contract (`C-04`).
- [x] T4 (VERIFY): run `cargo fmt --check`, scoped clippy, and targeted SDK tests.

## Test Tier Mapping

- Unit: frame parser unit tests (extended length + truncation).
- Integration: existing websocket route contract test.
- Regression: existing service API client tests for read event path remain green.
