# Spec: Issue #6313 - runtime_orchestration full/kolme mode handler extraction tranche 3

## Objective

Extract the `RuntimeModeKind::Full` and `RuntimeModeKind::KolmeLive` execution branches from
`crates/kamn-node/src/runtime_orchestration.rs` into a dedicated submodule so root orchestration
focuses on dispatch and high-level wiring.

## Inputs/Outputs

- Inputs:
  - current inline full/kolme branch logic inside `runtime_orchestration::execute`.
  - existing full-supervisor, daemon-phase, and runtime-policy submodules.
  - extraction boundary contracts and threshold checker.
- Outputs:
  - new submodule under `crates/kamn-node/src/runtime_orchestration/` owning full/kolme mode
    branch handlers.
  - root `runtime_orchestration.rs` delegating Full/Kolme arms to extracted functions.
  - updated extraction boundary assertions in contract tests.

## Boundaries/Non-goals

- In scope:
  - move only full/kolme execution branch bodies into a new submodule.
  - preserve runtime behavior, error paths, and reason-code semantics.
  - keep threshold policy non-failing (`GO` or `WARN`).
- Out of scope:
  - policy contract semantic changes.
  - CLI/arg parsing behavior changes.
  - CI/workflow/template/shell-surface changes.

## Failure modes

- FM-1: root file still contains inline Full/Kolme branch body logic after extraction.
- FM-2: visibility/wiring breakage prevents full or kolme execution path from compiling/running.
- FM-3: reason-code or fail-closed behavior drifts in full-supervisor stop flow or signer policy flow.
- FM-4: extraction threshold checker regresses to `NO-GO`.

## Acceptance criteria (testable booleans)

- AC-1: `runtime_orchestration.rs` declares a runtime mode-handler submodule and no longer
  contains inline Full/Kolme branch bodies.
- AC-2: `execute(...)` dispatches Full/Kolme modes via extracted handler functions with unchanged
  behavior and error semantics.
- AC-3: `cargo test -p kamn-node --test main_module_extraction_contract` passes with updated
  module boundary assertions.
- AC-4: `cargo test -p kamn-node` passes.
- AC-5: `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`
  returns non-failing policy decision (`GO` or `WARN`).

## Files to touch

- `specs/6313-runtime-orchestration-split-tranche-3-mode-handlers.md`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/runtime_orchestration/runtime_mode_handlers.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`

## Error semantics

- Preserve existing `ConfigError` variants and reason-code-bearing messages emitted through:
  - full-supervisor lane setup/teardown and stop-contract validation path.
  - kolme-live signer contract and key-source policy enforcement path.
- Preserve fail-closed behavior for invalid API/observability max-request contracts in full mode.

## Test plan

- RED:
  - update extraction contract assertions to require runtime mode-handler submodule and ban key
    inline Full/Kolme branch markers in root orchestrator.
  - run targeted contract test and verify failure.
- GREEN:
  - implement `runtime_mode_handlers` submodule with extracted Full/Kolme handlers.
  - rewire root match arms to delegate to extracted handlers.
- REFACTOR:
  - remove stale imports/duplication from root orchestrator.
- INTEGRATION:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
  - `cargo test -p kamn-node`
  - `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`
  - `cargo run -p kamn-node -- --role processor`

## Phase 6 integration evidence (to fill at close)

- `cargo test -p kamn-node --test main_module_extraction_contract`
- `cargo test -p kamn-node`
- `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`
- `cargo run -p kamn-node -- --role processor`
