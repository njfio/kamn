# Issue #5038 Spec

- Title: Subtask: M9 realtime delivery ordering, presence, and backpressure fail-closed checks
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5025` requires deterministic validation for M9 realtime behavior.
`kamn-core` already implements core dispatch/presence/backpressure surfaces, but
the issue-level contract is still missing explicit conformance coverage for
queue ordering observability and fail-closed markers in one scoped suite.

## Acceptance Criteria
- AC-1: Recipient queue ordering is exposed via a deterministic Rust contract
  and preserves dispatch insertion order for pending/deferred queues.
- AC-2: Presence visibility remains fail-closed until interaction/shared-escrow
  linkage is registered, and cross-owner queries/dispatches are denied with
  stable reason markers.
- AC-3: Backpressure thresholds emit deterministic warning/escrow-extension
  markers and queue-full reason markers.
- AC-4: Duplicate message identifiers are rejected fail-closed with typed
  errors.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add deterministic queue ordering snapshot contract to
  `data_layer_m9_realtime_delivery`.
- Extend `data_layer_m9_realtime_delivery` conformance tests with ordering and
  duplicate fail-closed assertions.
- Validate scoped/full regression and shell guardrail markers.

Out of scope:
- New dependencies/protocol/wire-format changes.
- CI workflow or shell-script surface changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Disconnected recipient with ordered dispatch IDs | Queue snapshot preserves insertion order in `pending_message_ids` |
| C-02 | AC-1 | Conformance | Queue-cap overflow with deferred dispatch IDs | Queue snapshot preserves insertion order in `deferred_message_ids` |
| C-03 | AC-2 | Functional | Presence query without relationship linkage | `PresenceVisibilityDenied` with stable reason marker |
| C-04 | AC-2 | Functional | Cross-owner dispatch/presence query | `OwnerScopeViolation` with stable reason marker |
| C-05 | AC-3 | Regression | Queue full beyond warning and sustained thresholds | Queue-full reason marker with warning/escalation toggles |
| C-06 | AC-4 | Regression | Reused message ID in pending/deferred state | `DuplicateMessageId` typed error |
| C-07 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5038.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5038.json`

## Success Metrics
- All `spec_c0x_*` tests in `data_layer_m9_realtime_delivery` pass with
  explicit ordering/presence/backpressure/duplicate guarantees.
- Shell-to-Rust ratio remains in-go and shell LOC remains below hard ceiling.
