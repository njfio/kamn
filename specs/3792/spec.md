# Issue #3792 Spec

- Title: Subtask: add notifications reconnect terminal taxonomy and drift contracts
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Notifications reconnect exhaustion currently provides deterministic text but lacks explicit terminal reason-code/taxonomy-version markers needed for policy enforcement and drift detection.

## Acceptance Criteria
- AC-1: Terminal reconnect failures emit deterministic reason-code and taxonomy-version markers.
- AC-2: Contract tests fail closed on reconnect terminal taxonomy drift.
- AC-3: Unit, Functional, Integration, and Regression evidence is present and passing (Performance N/A justified).

## Scope
In scope:
- `crates/kamn-kolme/src/notification_policy.rs`
- `crates/kamn-kolme/tests/notification_policy_contracts.rs`
- `crates/kamn-core/tests/kolme_runtime_commit_notifications.rs`
- `docs/architecture/kolme-runtime-commit.md`
- `crates/kamn-node/tests/kolme_runtime_commit_docs.rs`
- `specs/3792/{spec.md,plan.md,tasks.md}`

Out of scope:
- Non-notification transport taxonomy changes
- Websocket protocol feature expansion
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | reconnect exhaustion reason composition helper | Reason includes deterministic `reason_code` + `reason_taxonomy_version` markers |
| C-02 | AC-1 | Functional | notifications consumer reconnect exhaustion path | Terminal provider error reason contains deterministic taxonomy markers |
| C-03 | AC-2 | Regression | docs-contract test over runtime architecture reconnect taxonomy declarations | Missing taxonomy markers fail closed |
| C-04 | AC-2 | Regression | notification policy contract tests for reconnect taxonomy composition | Taxonomy drift in reason-code/version markers fails closed |
| C-05 | AC-3 | Integration | notifications consumer integration/websocket test suite | Existing integration behavior stays green with taxonomy marker updates |

## Test Mapping
- `cargo test -p kamn-kolme --test notification_policy_contracts`
- `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
- `cargo test -p kamn-node --test kolme_runtime_commit_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-kolme --all-targets -- -D warnings`
- `cargo clippy -p kamn-core --all-targets -- -D warnings`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3792.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3792.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3792.json`

## Success Metrics
- Notifications terminal reconnect failures include deterministic reason-code and taxonomy-version markers.
- Docs and tests fail closed on reconnect taxonomy drift.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
