# Spec: Issue #5901 - Replace Fake SDK Live Transport with Network-Backed Behavior

- Issue: #5901
- Status: Accepted
- Type: task
- Priority: P0
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`LiveTransportKamnClient` currently routes all `KamnAgent` operations through endpoint-keyed global `InMemoryKamnClient` state. This makes `TransportMode::Live` semantically false and hides real network/auth failures.

## Scope
In scope:
- Remove endpoint-scoped global in-memory registry from `crates/kamn-sdk/src/live.rs`.
- Implement network-backed live operations for Service API routes that exist today:
  - message send (`POST /v1/messages/send`)
  - agent profile lookup (`GET /v1/agents/{did}`)
- Keep `KamnTransport` and `KamnAgent` trait compatibility.
- Fail closed (`SdkError::NotImplemented`) for `KamnAgent` methods that cannot be mapped to existing Service API routes without synthetic local simulation.
- Add tests proving success against a local loopback HTTP server and failure when endpoint is unavailable.

Out of scope:
- Adding new Service API routes (for example, SDK-side agent registration/inbox drain endpoints).
- Converting SDK APIs to async.
- Multi-language SDK parity work.

## Acceptance Criteria
### AC-1 Live transport has no global in-memory simulation
Given `LiveTransportKamnClient` source,
When live transport state is inspected,
Then it does not use endpoint-keyed global `InMemoryKamnClient` registry state.

### AC-2 Live send executes real HTTP request path
Given a running local loopback Service API test server,
When `LiveTransportKamnClient::send` is called,
Then the client performs a network request and returns a deterministic `MessageId` derived from the service `message_id` value.

### AC-3 Live resolve/reputation execute real HTTP request paths
Given a running local loopback Service API test server,
When `resolve` and `get_reputation` are called,
Then the client queries `GET /v1/agents/{did}` and returns values derived from the response body.

### AC-4 Unavailable endpoint fails closed
Given an unreachable live endpoint,
When network-backed live operations are called,
Then they return `SdkError::TransportFailure` rather than synthetic pass behavior.

### AC-5 Unsupported live operations fail closed
Given `KamnAgent` methods with no current Service API route mapping,
When they are invoked on `LiveTransportKamnClient`,
Then they return explicit `SdkError::NotImplemented` and do not fallback to in-memory simulation.

## Conformance Cases
- C-01 (AC-1, Conformance): source contract test asserts `live.rs` does not reference `InMemoryKamnClient`, `OnceLock`, or endpoint registry maps.
- C-02 (AC-2, Functional/Integration): live send test runs against loopback HTTP contract server and asserts request path/method and deterministic ID mapping.
- C-03 (AC-3, Functional/Integration): live resolve/reputation test runs against loopback HTTP contract server and asserts returned DID + score mapping.
- C-04 (AC-4, Regression): live send/resolve against unreachable endpoint returns `SdkError::TransportFailure`.
- C-05 (AC-5, Functional): representative unsupported methods return `SdkError::NotImplemented`.

## Success Metrics / Observable Signals
- `crates/kamn-sdk/src/live.rs` no longer contains global registry + mutex-backed `InMemoryKamnClient` behavior.
- Live transport tests require actual loopback socket activity for covered operations.
- Live transport tests no longer pass by endpoint-keyed shared in-memory state.
