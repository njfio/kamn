# Issue #5091 Spec

- Title: Task: integrate M2 gateway DID validation with canonical AgentDid parser
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
`data_layer_m2_gateway_access.rs` currently validates DIDs with a local
`validate_kamn_did()` helper and raw string checks. This duplicates identity
validation logic from `did.rs` and allows M2-specific drift from canonical
`AgentDid` parsing behavior.

## Acceptance Criteria
- AC-1: M2 DID-auth session issuance validates requester identity through
  `AgentDid::parse` rather than local format-only checks.
- AC-2: M2 message-scope validation uses canonical `AgentDid` parsing for
  `sender_did` and `recipient_did`.
- AC-3: M2 ABAC agent-role authorization validates requester identity through
  `AgentDid::parse` while preserving fail-closed deterministic denials for
  non-agent roles.
- AC-4: M2 tests cover parser-backed accept/reject behavior, including malformed
  agent DID inputs that previously bypassed strict parser checks.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/tests/data_layer_m2_gateway_access.rs`
- `specs/5091/{spec.md,plan.md,tasks.md}`

Out of scope:
- Protocol/wire-format changes.
- New dependencies.
- M3/M7/M9/M10 integration-gap work.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Authenticate request with canonical agent DID | Session token issued deterministically |
| C-02 | AC-1 | Regression | Authenticate request with malformed agent DID (`kamn:did:agent:Bad`) | Fail-closed `InvalidDid` |
| C-03 | AC-2 | Conformance | Authorize scope with malformed sender/recipient agent DID | Fail-closed `InvalidDid` |
| C-04 | AC-3 | Conformance | Agent-role authorization with non-agent requester DID | Fail-closed `InvalidDid`; owner/auditor role behavior unchanged |
| C-05 | AC-4 | Regression | Run full M2 suite and crate regression | Deterministic pass with canonical parse enforcement |
| C-06 | AC-5 | Regression | Diff path audit | Zero shell/workflow/python/template changes |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m2_gateway_access`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- M2 uses canonical parser for agent DID validation paths.
- M2 suite demonstrates parser-backed strict rejection for malformed agent DIDs.
- Shell-to-Rust posture is improved/neutral with zero shell delta.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m2_gateway_access` failed with:
  - `spec_c02b_did_authentication_rejects_non_canonical_agent_did_shapes`
  - `spec_c03b_abac_rejects_non_canonical_agent_did_fields`
- GREEN: `cargo test -p kamn-core --test data_layer_m2_gateway_access` passed after
  integrating canonical parser checks.
- REGRESSION:
  - `cargo fmt --check` passed.
  - `cargo clippy -p kamn-core -- -D warnings` passed.
  - `cargo test -p kamn-core` passed.
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5091.json` -> `final_decision=GO`.
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5091.json` -> `final_decision=GO`.

## AC Verification
| AC | Result | Tests |
|---|---|---|
| AC-1 | ✅ | `spec_c01_did_authentication_issues_deterministic_bounded_session`; `spec_c02b_did_authentication_rejects_non_canonical_agent_did_shapes` |
| AC-2 | ✅ | `spec_c03b_abac_rejects_non_canonical_agent_did_fields` |
| AC-3 | ✅ | `spec_c03_abac_message_visibility_matrix_is_fail_closed_for_unrelated_requesters`; `spec_c03b_abac_rejects_non_canonical_agent_did_fields` |
| AC-4 | ✅ | `cargo test -p kamn-core --test data_layer_m2_gateway_access`; `cargo test -p kamn-core` |
| AC-5 | ✅ | `git diff --name-only` shows Rust + spec files only; shell guardrail checks `GO` |

## Shell Surface Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: +67
- shell_to_rust_ratio_delta_actual: improved
- shell_surface_ratio_target_status: improved
- shell_surface_mitigation_issue: None
