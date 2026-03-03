# Spec: Issue #6311 - runtime_orchestration policy contracts extraction tranche 2

## Objective

Extract runtime policy/contract helpers from
`crates/kamn-node/src/runtime_orchestration.rs` into a dedicated submodule so the root runtime
orchestrator stays focused on runtime-mode dispatch and integration wiring.

## Inputs/Outputs

- Inputs:
  - current inline policy helpers in `runtime_orchestration.rs`:
    - transport-profile policy/classification helpers
    - full-supervisor stop contract classifier/validator helpers
    - shutdown checkpoint reconciliation classifier/validator helpers
    - Kolme signer-policy helpers
  - extraction contract tests and threshold checker.
- Outputs:
  - new module under `crates/kamn-node/src/runtime_orchestration/` owning policy contracts.
  - root `runtime_orchestration.rs` declaring/delegating to the new module.
  - updated extraction contract assertions enforcing the new module boundary.

## Boundaries/Non-goals

- In scope:
  - move runtime policy/contract helper logic into a dedicated submodule.
  - preserve runtime behavior, error types, and reason-code strings.
  - keep extraction threshold policy non-failing (`WARN` or `GO`).
- Out of scope:
  - runtime mode behavior changes.
  - endpoint/API/observability protocol changes.
  - shell/python/workflow/template surface changes.

## Failure modes

- FM-1: root module still keeps inline policy helper definitions after extraction.
- FM-2: visibility/wiring regression breaks daemon/full/kolme runtime paths.
- FM-3: reason-code drift changes fail-closed semantics for policy violations.
- FM-4: extraction checker regresses to `NO-GO`.

## Acceptance criteria (testable booleans)

- AC-1: `runtime_orchestration.rs` declares a policy-contract submodule and no longer defines
  inline transport-profile, full-supervisor stop, shutdown checkpoint reconciliation, and Kolme
  signer-policy helper bodies.
- AC-2: runtime dispatch still compiles and executes through extracted policy helpers with
  unchanged fail-closed behavior.
- AC-3: `cargo test -p kamn-node --test main_module_extraction_contract` passes with updated
  module-boundary assertions.
- AC-4: `cargo test -p kamn-node` passes.
- AC-5: `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`
  returns non-failing policy decision (`GO` or `WARN`).

## Files to touch

- `specs/6311-runtime-orchestration-split-tranche-2-policy-contracts.md`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/runtime_orchestration/runtime_policy_contracts.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`

## Error semantics

- Preserve all existing `ConfigError` variants and message/reason-code payloads emitted by:
  - production transport profile policy violations
  - full-supervisor stop contract validation
  - shutdown checkpoint reconciliation validation
  - Kolme signer contract/key-source/fallback secret policy checks
- Preserve current fail-closed behavior on malformed/invalid policy inputs.

## Test plan

- RED:
  - update extraction contract assertions to require new policy-contract module and forbid key
    inline policy helper definitions in `runtime_orchestration.rs`.
  - run targeted extraction contract test and capture failing result.
- GREEN:
  - extract policy helper functions/constants into
    `runtime_orchestration/runtime_policy_contracts.rs`.
  - wire imports/re-exports from root module.
- REFACTOR:
  - tighten boundary surface and imports in root module; remove duplication/dead declarations.
- INTEGRATION:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
  - `cargo test -p kamn-node`
  - `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`
  - `cargo run -p kamn-node -- --role processor`

## Phase 6 integration evidence

- `cargo test -p kamn-node --test main_module_extraction_contract`:
  - pass (`12 passed, 0 failed`)
- `cargo test -p kamn-node`:
  - pass (`628 passed, 0 failed`)
- `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --output-json ...`:
  - `status=pass`
  - `policy_decision=GO`
  - `line_count=536`
  - `warn_line_count=950`
  - `fail_line_count=1100`
- `cargo run -p kamn-node -- --role processor`:
  - pass (runtime bootstrap path executed and exited cleanly)

## Deviations

- None.
