# Spec: Issue #6309 - runtime_orchestration full-supervisor extraction tranche 1

## Objective

Extract the full-supervisor lane/probe subsystem from
`crates/kamn-node/src/runtime_orchestration.rs` into a dedicated submodule so the root runtime
orchestrator remains focused on policy and runtime-mode dispatch.

## Inputs/Outputs

- Inputs:
  - current `runtime_orchestration.rs` inline full-supervisor lane/probe structs, helpers, and tests.
  - runtime orchestration contracts/tests and extraction-threshold checker.
- Outputs:
  - new submodule under `crates/kamn-node/src/runtime_orchestration/` owning full-supervisor
    lane/probe logic.
  - root `runtime_orchestration.rs` delegating to extracted module functions/types.
  - updated extraction contract assertions covering the new boundary.

## Boundaries/Non-goals

- In scope:
  - move full-supervisor lane/probe subsystem and its unit tests out of root file.
  - keep public behavior and reason-code semantics unchanged.
  - make checker output non-failing (`WARN` or `GO`).
- Out of scope:
  - runtime-mode semantic changes.
  - API/observability route contract changes.
  - CI workflow/shell/python/template changes.

## Failure modes

- FM-1: root file still contains full-supervisor inline helpers after extraction.
- FM-2: helper visibility changes break runtime execution wiring.
- FM-3: extracted tests are not wired and probe/regression coverage silently drops.
- FM-4: threshold checker remains `NO-GO` (line count above fail threshold).

## Acceptance criteria (testable booleans)

- AC-1: `runtime_orchestration.rs` declares a full-supervisor submodule and no longer defines inline
  full-supervisor probe/lane helpers (`run_full_supervisor_http_probe`, lane start/finish helpers,
  and lane structs).
- AC-2: full-supervisor branch in `execute(...)` still compiles and executes through extracted
  module functions with unchanged fail-closed behavior.
- AC-3: `cargo test -p kamn-node --test main_module_extraction_contract` passes with new module
  boundary assertions.
- AC-4: `cargo test -p kamn-node` passes.
- AC-5: `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`
  returns non-failing policy decision (`GO` or `WARN`).

## Files to touch

- `specs/6309-runtime-orchestration-split-tranche-1.md`
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/runtime_orchestration/full_supervisor.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`

## Error semantics

- Preserve existing `ConfigError::RuntimeDaemonLifecycle` reason-code and detail formatting for
  full-supervisor lane/probe paths.
- Preserve probe fail-closed classification on non-2xx or malformed responses.
- Preserve graceful shutdown/stop contract validations and logging semantics.

## Test plan

- RED:
  - add extraction contract assertions requiring a dedicated full-supervisor submodule and banning
    key inline helpers in `runtime_orchestration.rs`.
  - run targeted contract test and capture failure.
- GREEN:
  - extract full-supervisor lane/probe subsystem into
    `runtime_orchestration/full_supervisor.rs`.
  - wire imports/calls from root module.
- REFACTOR:
  - normalize helper boundaries and remove dead imports in root module.
- INTEGRATION:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
  - `cargo test -p kamn-node`
  - `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`

## Phase 6 integration evidence

- `cargo test -p kamn-node --test main_module_extraction_contract`:
  - pass (`12 passed, 0 failed`)
- `cargo test -p kamn-node --test runtime_output_contract`:
  - pass (`5 passed, 0 failed`)
- `cargo test -p kamn-node`:
  - pass (`628 passed, 0 failed`)
- `bash scripts/ci/check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --output-json ...`:
  - `status=warn`
  - `policy_decision=WARN`
  - `line_count=1024`
  - `fail_line_count=1100`
- `cargo run -p kamn-node -- --role processor`:
  - pass (bootstrap runtime executed and exited cleanly)

## Deviations

- None.
