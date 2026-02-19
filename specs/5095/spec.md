# Issue #5095 Spec

- Title: Task: complete M2 AgentDid type integration for auth and ABAC contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Issue `#5091` switched key M2 validations to `AgentDid::parse`, but internal M2
contracts still propagate identities as raw `String`. This leaves type-safety debt
and increases risk of future untyped paths diverging from canonical DID semantics.

## Acceptance Criteria
- AC-1: M2 introduces typed validated auth/scope contracts using `AgentDid` for
  requester/sender/recipient identity fields.
- AC-2: String-based public request/scope inputs are converted at boundaries into
  typed contracts with fail-closed deterministic `InvalidDid` errors.
- AC-3: Auth and ABAC internal evaluation paths consume typed contracts (single
  parse-at-boundary behavior).
- AC-4: Existing reason-marker behavior remains deterministic and backward
  compatible for current public APIs.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/tests/data_layer_m2_gateway_access.rs`
- `specs/5095/{spec.md,plan.md,tasks.md}`

Out of scope:
- Protocol/wire-format changes.
- New dependencies.
- Other module integration gaps.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Convert valid auth request to typed contract | Typed conversion succeeds with `AgentDid` fields |
| C-02 | AC-2 | Regression | Convert malformed auth requester DID | Fail-closed `InvalidDid` |
| C-03 | AC-1/AC-2 | Conformance | Convert valid/invalid message scope to typed scope contract | Valid scope converts; malformed sender/recipient fails closed |
| C-04 | AC-3 | Functional | Run auth + ABAC paths with valid typed-boundary inputs | Existing behavior preserved, internal typed path used |
| C-05 | AC-4 | Regression | Existing reason-marker assertions | Stable reason-code outputs remain unchanged |
| C-06 | AC-5 | Regression | Diff audit + shell guardrails | Zero shell-surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m2_gateway_access`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5095.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5095.json`

## Success Metrics
- M2 identity validation is typed internally using `AgentDid` contracts.
- Boundary conversion tests cover deterministic success/failure paths.
- Shell-to-Rust posture remains improved/neutral with zero shell delta.
