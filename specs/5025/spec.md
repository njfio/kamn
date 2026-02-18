# Issue #5025 Spec

- Title: Task: M9 deliver realtime delivery pipeline, presence, and deterministic backpressure
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M9 requires deterministic contracts for realtime delivery acknowledgements,
presence visibility scoping, and backpressure escalation markers. Existing
runtime code contains queue/backpressure internals, but there is no dedicated
owner-scoped M9 contract surface that unifies delivery ACK semantics, presence
policy checks, and sustained backpressure escalation outcomes.

PRD mapping:
- Section 10.1 (WebSocket/SSE delivery + ACK/requeue contract)
- Section 10.2 (presence system and scoped visibility)
- Section 10.4 (backpressure queue cap, warning after 5m, sustained after 1h)
- Milestone table M9 deliverables (delivery pipeline + presence + queue behavior)

## Acceptance Criteria
- AC-1: Realtime dispatch contract deterministically returns `Delivered` ACK for
  connected recipients without backlog and `Queued` ACK when delivery must be
  deferred.
- AC-2: Presence visibility contract is fail-closed and only allows requester
  visibility for counterparties with prior interaction or shared escrow linkage.
- AC-3: Backpressure contract enforces deterministic per-recipient queue cap and
  emits escalation markers at >5m (warning) and >1h (escrow timeout extension).
- AC-4: Cross-owner dispatch and presence operations are denied fail-closed with
  stable reason markers.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M9 module in `kamn-core` for owner-scoped dispatch ACKs,
  presence visibility queries, and deterministic backpressure escalation.
- Conformance tests for ACK behavior, scoped presence access, queue-cap
  saturation behavior, and sustained-pressure markers.
- Public API exports for downstream M10/M11 integration lanes.

Out of scope:
- Live websocket/SSE runtime wiring and gateway network transport.
- New shell/python/workflow/template orchestration.
- New dependencies or wire/protocol format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Dispatch to connected recipient with empty backlog | `Delivered` ACK with stable reason marker and zero queue depth |
| C-02 | AC-2 | Conformance | Query target presence before/after prior interaction registration | Query is denied before linkage and allowed after linkage |
| C-03 | AC-3 | Conformance | Saturate recipient queue and dispatch beyond 5m and 1h thresholds | Deterministic warning + escalation flags at policy boundaries |
| C-04 | AC-4 | Regression | Dispatch/presence queries with mismatched owner scope | Owner-scope violation returned with stable reason marker |
| C-05 | AC-3/AC-4 | Regression | Queue-full dispatch while disconnected | `Queued` ACK persists and deferred counters remain deterministic |
| C-06 | AC-5 | Regression | Inspect issue diff paths | No shell/python/workflow/template path changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- M9 contracts are exported via `kamn_core` for downstream integration lanes.
- All ACs map to passing `spec_c0x_*` conformance tests.
- Shell-to-Rust ratio direction remains improved/neutral through Rust-only changes.

## Verification
| AC | Result | Tests/Evidence |
|---|---|---|
| AC-1 | ✅ | `spec_c01_connected_recipient_without_backlog_receives_delivered_ack`, `spec_c05_queue_full_dispatch_keeps_pending_cap_and_increments_deferred_counter` |
| AC-2 | ✅ | `spec_c02_presence_query_is_denied_until_relationship_linkage_is_registered` |
| AC-3 | ✅ | `spec_c03_backpressure_thresholds_emit_warning_and_sustained_escalation_markers` |
| AC-4 | ✅ | `spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed` |
| AC-5 | ✅ | `git diff --name-only` confirms no shell/python/workflow/template path changes |

Executed commands:
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
