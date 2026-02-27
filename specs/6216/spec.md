# Spec: Issue 6216 - Publish Channel/Task/Bridge Lifecycle Events via WebSocket Fanout

- Issue: #6216
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: networking

## Problem Statement

WebSocket fanout currently emits `service-api.message.created` events only.
R59 flagged missing lifecycle fanout coverage for channels, tasks, and bridges.

## Scope

In scope:
1. Add websocket fanout events for channel creation.
2. Add websocket fanout events for task submission and task transitions.
3. Add websocket fanout events for bridge submission and forwarding.
4. Wire publish calls from service API middleware success paths.

Out of scope:
1. New websocket endpoints or protocol version changes.
2. Changes to presence-mode websocket projection.

## Acceptance Criteria

### AC-1 Channel Lifecycle Events Published
Given successful `POST /v1/channels/create`,
When a websocket subscriber is connected,
Then a `service-api.channel.created` event is published.

### AC-2 Task Lifecycle Events Published
Given successful task create/transition operations,
When websocket fanout runs,
Then `service-api.task.submitted` and transition events are published.

### AC-3 Bridge Lifecycle Events Published
Given successful bridge submit/forward operations,
When websocket fanout runs,
Then `service-api.bridge.submitted` and `service-api.bridge.forwarded` events are published.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6216_websocket_fanout_publishes_channel_task_bridge_lifecycle_events`
- C-02 (AC-2, Unit): `tests::regression_issue_6216_websocket_fanout_publishes_channel_task_bridge_lifecycle_events`
- C-03 (AC-3, Unit): `tests::regression_issue_6216_websocket_fanout_publishes_channel_task_bridge_lifecycle_events`
