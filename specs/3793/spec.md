# Issue #3793 Spec

- Title: Subtask: implement deterministic reconnect pacing schedule for notifications consumer
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Kolme notifications consumer reconnect attempts are budget bounded but currently immediate; deterministic reconnect pacing is required to avoid tight-loop retry behavior under endpoint flapping and to keep terminal attempt-budget outcomes deterministic.

## Acceptance Criteria
- AC-1: Notifications consumer reconnect loop applies deterministic backoff pacing with bounded increments/cap.
- AC-2: Attempt budget exhaustion remains deterministic and surfaced with stable terminal outcome mapping.
- AC-3: Unit, Functional, Integration, Regression, and bounded Performance evidence is present and passing.
- AC-4: Runtime architecture documentation declares reconnect pacing policy markers and remains contract-tested.

## Scope
In scope:
- `crates/kamn-core/src/kolme_runtime_commit/notifications_consumer.rs`
- `crates/kamn-core/tests/kolme_runtime_commit_notifications.rs`
- `docs/architecture/kolme-runtime-commit.md`
- `crates/kamn-node/tests/kolme_runtime_commit_docs.rs`
- `specs/3793/{spec.md,plan.md,tasks.md}`

Out of scope:
- Websocket protocol feature changes
- Non-notification transport taxonomy changes (covered by `#3792`)
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | deterministic reconnect pacing helper across attempt range | Backoff schedule is deterministic and capped |
| C-02 | AC-1 | Functional | repeated connection/read failures before exhaustion | Loop applies non-zero reconnect pacing before subsequent attempts |
| C-03 | AC-2 | Regression | retry exhaustion behavior tests | Terminal exhaustion remains deterministic/stable |
| C-04 | AC-3 | Integration | websocket connector notification path under reconnect behavior | Consumer still reconnects and decodes valid events correctly |
| C-05 | AC-3 | Performance | reconnect pacing budget test | Bounded delay and attempt budget remain within expected upper bound |
| C-06 | AC-4 | Regression | docs-contract assertions over runtime architecture doc markers | Missing pacing markers fail closed |

## Test Mapping
- `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
- `cargo test -p kamn-node --test kolme_runtime_commit_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --all-targets -- -D warnings`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3793.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3793.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3793.json`

## Success Metrics
- Notifications reconnect pacing is deterministic, bounded, and test-verified.
- Retry exhaustion remains deterministic and stable.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
