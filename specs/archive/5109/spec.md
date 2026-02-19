# Issue #5109 Spec

- Title: Task: integrate M9 realtime delivery DID contracts with canonical parser taxonomy
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M9 still uses local DID string validation and reports coarse `InvalidDid(String)` errors. This duplicates canonical DID parsing and does not provide field-scoped deterministic taxonomy for owner/sender/recipient/requester DID failures.

## Acceptance Criteria
- AC-1: M9 owner DID validation uses canonical `KamnDid::parse` and agent DID validation uses canonical `AgentDid::parse`.
- AC-2: Invalid DID failures in M9 return deterministic field-scoped reason markers.
- AC-3: Existing M9 dispatch, presence, queue, and runtime backpressure semantics remain backward compatible except for enriched invalid-DID taxonomy.
- AC-4: M9 conformance tests cover invalid DID rejection in requester-owner, sender, recipient, and requester-agent paths.
- AC-5: Shell/workflow/python/template LOC remain unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5109/{spec.md,plan.md,tasks.md}`

Out of scope:
- Dependency changes
- Runtime behavior changes outside DID parser/taxonomy paths
- Shell/python/workflow/template changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Valid owner/agent DID inputs across M9 presence/dispatch/backpressure flows | Canonical parser validation succeeds and behavior remains green |
| C-02 | AC-2 | Conformance | Invalid requester owner DID | `InvalidDid` with requester-owner field + stable reason code |
| C-03 | AC-2 | Conformance | Invalid sender/recipient/requester-agent DID shapes | `InvalidDid` with field-specific reason codes |
| C-04 | AC-3 | Regression | Existing M9 `spec_c01..spec_c13` behavior corpus | Existing delivery/presence/backpressure semantics remain green |
| C-05 | AC-5 | Regression | Shell guardrails | Zero shell delta; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5109.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5109.json`

## Success Metrics
- M9 DID validation paths consume canonical parser contracts.
- Invalid DID taxonomy is deterministic and field-scoped.
- Shell governance posture is unchanged or improved.
