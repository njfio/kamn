# Issue #5107 Spec

- Title: Task: integrate M2 gateway DID contracts with canonical AgentDid parsing
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M2 still carries local DID validation helpers and does not expose deterministic field-scoped DID parse failures for requester/sender/recipient inputs. This leaves M2 DID validation partially detached from canonical DID contracts and blurs error taxonomy when failures occur.

## Acceptance Criteria
- AC-1: M2 requester/sender/recipient DID validation routes through canonical DID parser contracts (`AgentDid` + canonical KAMN DID parser).
- AC-2: Invalid DID errors fail closed with deterministic M2 reason taxonomy that identifies DID field context.
- AC-3: Existing M2 auth, ABAC, negative-matrix, and access-audit behavior remains backward compatible except for enriched DID error taxonomy.
- AC-4: Conformance tests cover invalid DID rejection at requester/sender/recipient positions.
- AC-5: Shell/workflow/python/template LOC remain unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/did.rs`
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/tests/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5107/{spec.md,plan.md,tasks.md}`

Out of scope:
- Dependency changes
- Wire/protocol changes outside M2 parsing and taxonomy
- Shell/python/workflow/template changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | M2 auth and ABAC requests with valid canonical DIDs | Typed validation succeeds via canonical DID parsers |
| C-02 | AC-2 | Conformance | Invalid requester DID in auth flow | Fail-closed M2 invalid DID with deterministic field reason code |
| C-03 | AC-2 | Conformance | Invalid sender/recipient DID in message scope flow | Fail-closed M2 invalid DID with deterministic field reason code |
| C-04 | AC-3 | Regression | Existing M2 `spec_c01..spec_c06` behavior paths | Existing auth/ABAC/audit semantics remain green |
| C-05 | AC-5 | Regression | Shell guardrails | Zero shell delta; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m2_gateway_access`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5107.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5107.json`

## Success Metrics
- M2 requester/sender/recipient DID validation consistently reuses canonical DID parsing.
- DID parse failures are deterministic and field-scoped.
- Shell governance posture is unchanged or improved.
