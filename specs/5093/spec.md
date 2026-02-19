# Issue #5093 Spec

- Title: Task: integrate M9 realtime delivery with channel membership and anti-spam controls
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
`data_layer_m9_realtime_delivery.rs` currently enforces owner-scope presence/queue
policy but does not consume existing `ChannelStore` membership rules or
`AntiSpamEngine` admission decisions. This leaves realtime delivery policy in a
parallel model disconnected from canonical channel and admission controls.

## Acceptance Criteria
- AC-1: M9 provides channel-membership-aware dispatch authorization using
  `ChannelStore` and fails closed when sender or recipient is not a channel
  member.
- AC-2: M9 provides anti-spam-gated dispatch authorization using
  `AntiSpamEngine` and maps rejections to stable M9 reason markers.
- AC-3: M9 provides a combined controls dispatch path (owner scope + channel
  membership + anti-spam + dispatch queue semantics) with deterministic results.
- AC-4: Existing non-channel M9 dispatch/presence behavior remains deterministic
  and unchanged.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/src/lib.rs` (exports for additive M9 APIs/constants)
- `specs/5093/{spec.md,plan.md,tasks.md}`

Out of scope:
- New dependencies, protocol changes, or runtime websocket wiring.
- Shell/workflow/template changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Channel dispatch where both sender+recipient are channel members | Authorization passes and dispatch outcome remains deterministic |
| C-02 | AC-1 | Regression | Channel dispatch where sender or recipient is not channel member | Fail-closed channel membership deny with stable reason marker |
| C-03 | AC-2 | Conformance | Anti-spam insufficient-deposit/rate-limit/duplicate rejection paths | Fail-closed anti-spam deny with stable mapped M9 reason marker |
| C-04 | AC-3 | Functional | Combined controls path with valid membership + accepted anti-spam | Dispatch succeeds and returns normal M9 ack outcome |
| C-05 | AC-3 | Functional | Combined controls path with anti-spam denial | Dispatch denied before queue mutation with stable reason marker |
| C-06 | AC-4 | Regression | Existing M9 core tests | Existing deterministic behavior remains green |
| C-07 | AC-5 | Regression | Diff path audit + shell guardrail checks | Zero shell-surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5093.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5093.json`

## Success Metrics
- M9 references and enforces both channel membership and anti-spam controls.
- New integration tests prove deterministic allow/deny behavior.
- Shell-to-Rust posture remains improved/neutral with zero shell delta.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` failed with unresolved
  additive APIs/constants (`authorize_channel_dispatch`, `dispatch_message_with_controls`,
  `DataLayerM9ChannelDispatchAuthorizationRequest`, anti-spam/channel reason markers, and
  new error variants).
- GREEN: same scoped command passed after implementing additive M9 controls integration.
- REGRESSION:
  - `cargo fmt --check` passed.
  - `cargo clippy -p kamn-core -- -D warnings` passed.
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` passed.
  - `cargo test -p kamn-core` passed.
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5093.json` -> `final_decision=GO`.
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5093.json` -> `final_decision=GO`.

## AC Verification
| AC | Result | Tests |
|---|---|---|
| AC-1 | ✅ | `spec_c09_channel_dispatch_requires_sender_and_recipient_membership` |
| AC-2 | ✅ | `spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes` |
| AC-3 | ✅ | `spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts` |
| AC-4 | ✅ | Existing `spec_c01`..`spec_c08` in `data_layer_m9_realtime_delivery` + `cargo test -p kamn-core` |
| AC-5 | ✅ | `git diff --name-only` shows Rust+spec files only; shell guardrails `GO` |

## Shell Surface Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: +385
- shell_to_rust_ratio_delta_actual: improved
- shell_surface_ratio_target_status: improved
- shell_surface_mitigation_issue: None
