# Spec: Issue 6190 - SDK WebSocket Extended-Length Frame Support

- Issue: #6190
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P0
- Area: sdk

## Problem Statement

`ServiceApiClient::read_event_once` currently parses WebSocket frames using only
7-bit payload length (`frame[1] & 0x7f`) and assumes payload starts at byte 2.
RFC 6455 frames with payload length indicators `126` and `127` are therefore parsed
incorrectly and truncated.

## Scope

In scope:
1. Support unmasked server-to-client text frames with RFC 6455 payload lengths:
   - inline (0..=125),
   - extended 16-bit (`126`),
   - extended 64-bit (`127`).
2. Preserve existing fail-closed semantics for masked, truncated, or unsupported frames.
3. Add unit coverage for extended length parsing.

Out of scope:
1. Fragmented WebSocket frame reassembly.
2. Binary opcode support.
3. Multi-event streaming changes.

## Acceptance Criteria

### AC-1 RFC6455 Length Modes
Given a valid unmasked text frame,
When payload length marker is `125`, `126`, or `127`,
Then payload bytes are parsed from the correct offset and decoded without truncation.

### AC-2 Fail-Closed Guards
Given truncated or malformed extended-length frame headers,
When parsing is attempted,
Then parsing fails with deterministic transport failure reason markers.

### AC-3 Regression Safety
Given existing integration route tests for `read_event_once`,
When parser changes land,
Then existing WebSocket route contract tests remain green.

## Conformance Cases

- C-01 (AC-1, Unit): parse 16-bit extended length text frame (`126`) and recover full payload.
- C-02 (AC-1, Unit): parse 64-bit extended length text frame (`127`) and recover full payload.
- C-03 (AC-2, Unit): truncated extended-length header/payload fails with deterministic error.
- C-04 (AC-3, Integration): existing SDK service API websocket integration contract remains green.

## Success Signals

1. `read_event_once` no longer assumes payload starts at byte 2 for all frames.
2. New unit tests for 126/127 length modes pass.
3. Existing websocket route contract test remains passing.
