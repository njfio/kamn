# Spec: Issue #6111 - SDK WebSocket extended payload length support

- Issue: #6111
- Status: Reviewed
- Type: task
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6098

## Problem Statement
`ServiceApiClient::read_event_once` only decodes WebSocket payload lengths from the 7-bit inline length field. RFC 6455 frames with length markers `126` and `127` are not decoded, so frames larger than 125 bytes are treated as truncated/corrupted.

## Scope
In scope:
- Add RFC 6455 payload-length decoding support for server-to-client unmasked text frames using 7-bit, 16-bit (`126`), and 64-bit (`127`) length markers.
- Preserve fail-closed behavior for masked frames, unsupported opcodes, overflow lengths, and truncated payloads.
- Add RED/GREEN regression tests for frame-length decoding and an integration test for extended frame delivery.

Out of scope:
- Fragmented continuation frame reassembly.
- Client masking behavior (server response frames remain unmasked).
- Multi-event stream parsing beyond the single-frame contract of `read_event_once`.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: SDK successfully decodes unmasked text frames using inline payload lengths `0..=125`.
- AC-2: SDK successfully decodes unmasked text frames using 16-bit extended payload lengths (`126` marker).
- AC-3: SDK successfully decodes unmasked text frames using 64-bit extended payload lengths (`127` marker) when value fits platform `usize`.
- AC-4: SDK fails closed with deterministic transport errors for truncated extended headers/payloads and for payload-length overflow.

## Conformance Cases
- C-01 (Unit, AC-1): parser returns payload for inline-length unmasked text frame.
- C-02 (Unit, AC-2): parser returns payload for 16-bit extended-length frame (>125 bytes).
- C-03 (Unit, AC-3): parser returns payload for 64-bit extended-length frame.
- C-04 (Unit, AC-4): parser rejects truncated extended-length header and truncated payload.
- C-05 (Integration, AC-2): `ServiceApiClient::read_event_once` reads a live websocket event frame with payload length >125 bytes.

## Success Metrics / Observable Signals
- RED tests fail before decoder change on extended payload cases.
- GREEN tests pass for unit parser conformance and integration websocket event read.
- `cargo test -p kamn-sdk service` and `cargo test -p kamn-sdk service_api_client::integration_service_api_client_reads_websocket_event_frame_extended_length` pass.
