# Spec: Issue #6043 - Add server-side WebSocket heartbeat and stale-session timeout

- Issue: #6043
- Status: Implemented
- Type: task
- Priority: P1
- Area: networking
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5973

## Problem Statement
`service_api_endpoint` WebSocket streams currently respond to incoming ping/pong frames but do not initiate server-side heartbeat pings or enforce stale-session timeouts when clients disappear without close frames.

## Scope
In scope:
- Add server-side heartbeat ping cadence to websocket stream loop.
- Add stale-session timeout close behavior when no client activity/pong is observed.
- Add deterministic tests for heartbeat and timeout behavior.

Out of scope:
- WebSocket protocol/schema redesign.
- Fanout/routing behavior changes outside keepalive lifecycle.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Active websocket sessions receive periodic server-initiated ping frames.
- AC-2: Sessions close deterministically after configured heartbeat timeout without inbound traffic.
- AC-3: Existing event streaming remains compatible for active clients.

## Conformance Cases
- C-01 (Conformance, AC-1): Stream loop emits ping at configured interval while connection remains open.
- C-02 (Regression, AC-2): Stale session without incoming frames times out and closes.
- C-03 (Functional, AC-3): Event payload streaming continues to deliver route events to active clients.

## Success Metrics / Observable Signals
- New websocket keepalive tests pass in `kamn-node`.
- Existing websocket/service-api route tests remain green.
