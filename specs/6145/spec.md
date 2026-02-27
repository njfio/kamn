# Spec: Issue #6145 - Task: [X-07] Wire multi-node P2P transport into message delivery

- Issue: #6145
- Status: Reviewed
- Type: task
- Priority: P1
- Area: networking
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6099

## Problem Statement
`kamn-node` daemon relay currently forwards queued `/v1/messages/send` entries only through HTTP
recipient route maps. That leaves the live P2P transport path unwired for message delivery, so
multi-node relay cannot run over the transport profile used by production runtime policy.

## Scope
In scope:
- Add optional daemon relay P2P wiring that can forward queued relay entries through
  `kamn_core` transport and ingest inbound relay frames into local service-api message state.
- Keep existing HTTP relay forwarding as compatibility fallback when P2P is not configured or
  cannot route a recipient.
- Add regression/conformance coverage for P2P success and failure paths.
- Update lifecycle/spec artifacts for deterministic AC-to-test mapping.

Out of scope:
- Protocol redesign beyond relay-path transport selection.
- Unrelated runtime orchestration refactors outside daemon relay handling.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Daemon relay tick loop supports an optional P2P transport path for outbound relay entries,
  keyed by recipient DID → peer-id mapping, without removing existing HTTP behavior.
- AC-2: Daemon relay tick loop drains inbound P2P relay frames and upserts relayed message state
  into local service-api persistence.
- AC-3: If outbound P2P delivery fails for an entry, daemon requeues that entry deterministically
  and does not project false relayed status.
- AC-4: Regression/conformance tests cover outbound P2P success + failure and preserve existing
  HTTP/no-route behavior.

## Conformance Cases
- C-01 (Integration/Conformance, AC-1/AC-2): Two deterministic daemon relay nodes configured for
  shared P2P topic deliver a queued relay message from sender spool to recipient state via P2P.
- C-02 (Regression, AC-3): Recipient peer unavailable path requeues relay entry and increments
  processing error count without projecting sender status to `relayed`.
- C-03 (Functional/Conformance, AC-4): Existing no-route/HTTP relay behavior remains deterministic
  when P2P relay config is absent.

## Success Metrics / Observable Signals
- Targeted R59 finding `X-07` no longer appears as unresolved in follow-up review docs.
- Required scoped test commands pass in CI and local verification runs.
- Closure comment includes deterministic evidence links and tier coverage summary.
