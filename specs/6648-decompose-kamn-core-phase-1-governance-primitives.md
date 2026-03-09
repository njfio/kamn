# 6648 Decompose KAMN Core Phase 1 Governance Primitives

## Objective

Extract the first executable governance slice from `kamn-core` into a new `kamn-governance` crate, proving the target crate graph is workable without pulling multiple unrelated domains into the same move.

## Inputs/Outputs

- Inputs:
  - `docs/architecture/kamn-core-target-crate-graph.md`
  - Current `kamn-core` governance-related modules
  - Existing `kamn-core` governance tests and public re-exports
- Outputs:
  - New `crates/kamn-governance` crate
  - `kamn-core` dependency on `kamn-governance`
  - Compatibility shims in `kamn-core` for extracted modules
  - Passing contract/integration tests through existing `kamn-core` entrypoints

## Boundaries/Non-goals

- Extract only governance primitives in this phase: `governance_workflow`, `operator_binding`, and `operator_actions`
- Do not extract `operator_dashboard_api` or `operator_dashboard_ui` in this issue; they still depend on task, escrow, message, and reputation projections owned by other domains
- Do not rename public `kamn-core` module paths in this issue
- Do not move unrelated runtime, escrow, or compliance modules

## Failure Modes

- The new crate depends back on `kamn-core`, violating the approved target graph
- `kamn-core` public imports break because compatibility shims are missing
- Governance tests stop exercising the real `kamn-core` entrypoints after extraction
- The extracted crate omits a required governance primitive (`governance_workflow`, `operator_binding`, or `operator_actions`)
- Dashboard modules are moved prematurely and drag cross-domain dependencies into the extraction

## Acceptance Criteria

- [x] A new `kamn-governance` crate exists in the workspace
- [x] `governance_workflow`, `operator_binding`, and `operator_actions` are owned by `kamn-governance`
- [x] `kamn-governance` does not depend on `kamn-core`
- [x] `kamn-core` depends on `kamn-governance` and preserves stable public module/re-export paths through compatibility shims
- [x] Existing governance/operator tests continue to pass through `kamn-core`
- [x] Contract coverage proves dashboards remain in `kamn-core` for this phase
- [x] `kamn-core` loses the moved implementation LOC from the extracted modules

## Files To Touch

- `specs/6648-decompose-kamn-core-phase-1-governance-primitives.md`
- `Cargo.toml`
- `crates/kamn-core/Cargo.toml`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/src/governance_workflow.rs`
- `crates/kamn-core/src/operator_binding.rs`
- `crates/kamn-core/src/operator_actions.rs`
- `crates/kamn-governance/**`
- `crates/kamn-core/tests/kamn_core_governance_phase1_contract.rs`
- `docs/architecture/README.md`
- `docs/architecture/kamn-governance.md`

## Error Semantics

- Missing governance modules or missing compatibility shims must fail deterministically in contract tests
- `kamn-governance` must remain a leaf/domain crate and fail the issue if a `kamn-core` dependency is introduced
- `kamn-core` governance public APIs must continue to hard-fail with the same typed errors after extraction

## Test Plan

- Run `cargo test -p kamn-governance --test governance_workflow_internal -- --nocapture`
- Run `cargo test -p kamn-core --test kamn_core_governance_phase1_contract -- --nocapture`
- Run `cargo test -p kamn-core --test governance_workflow -- --nocapture`
- Run `cargo test -p kamn-core --test operator_permissioned_actions -- --nocapture`
- Run `cargo test -p kamn-core --test operator_dashboard_ui -- --nocapture`
- Run `cargo test -p kamn-core --test operator_dashboard_api -- --nocapture`
- Run `cargo test -p kamn-core --test governance_workflow_docs -- --nocapture`
- Run `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6648-touched-size.json`

## Phase 6 Evidence

- `kamn-core` compatibility shims in `src/governance_workflow.rs`, `src/operator_binding.rs`, and `src/operator_actions.rs` continue to drive the extracted governance implementations through the original public module paths.
- Cross-domain dashboard entrypoints stayed in `kamn-core` and passed their existing integration-style tests after extraction.
- The touched-Rust size gate passed after splitting the new `kamn-governance` modules into bounded files and shrinking the branch-touched contract tests below the function limit.

## Deviations

- `kamn-governance` does not depend on `kamn-types` in Phase 1. The first attempt recreated a crate cycle through `kamn-core`, so the extracted crate now uses private DID parsing helpers until the broader `kamn-types` inversion work lands.
