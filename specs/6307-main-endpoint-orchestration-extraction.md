# Spec: Issue #6307 - main.rs endpoint-serving orchestration extraction

## Objective

Extract post-runtime endpoint-serving orchestration from `crates/kamn-node/src/main.rs` into a
dedicated module so the entrypoint remains focused on CLI/bootstrap flow while preserving runtime
behavior.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-node/src/main.rs` inline endpoint-serving logic and helpers.
  - existing endpoint config/snapshot/server helpers in `service_api_endpoint` and
    `observability_endpoint` modules.
  - existing `kamn-node` extraction/runtime contract tests.
- Outputs:
  - new module owning endpoint-serving path classification + serve orchestration.
  - `main.rs` delegating endpoint-serving responsibility via module boundary.
  - new/updated extraction contract tests guarding the boundary.

## Boundaries/Non-goals

- In scope:
  - extraction of service/observability endpoint serving logic from `run()`.
  - extraction of endpoint-path classification helpers from `main.rs`.
  - contract tests asserting delegation markers.
- Out of scope:
  - runtime mode semantics changes.
  - API/observability wire-format or response contract changes.
  - shell/python/workflow/template surface changes.

## Failure Modes

- FM-1: endpoint-serving behavior diverges (missing logs, serving wrong path, wrong error mapping).
- FM-2: `main.rs` retains inlined helper logic after extraction.
- FM-3: extraction boundary regresses in future edits without contract coverage.

## Acceptance Criteria

- AC-1: `main.rs` no longer defines
  `classify_service_api_endpoint_runtime_path` and
  `should_skip_observability_endpoint_for_full_supervisor`.
- AC-2: endpoint-serving orchestration is delegated through a dedicated module function from
  `run()`.
- AC-3: `main.rs` line count is lower than baseline 773.
- AC-4: `cargo test -p kamn-node --test main_module_extraction_contract` passes with explicit
  boundary assertions for the new module.
- AC-5: `cargo test -p kamn-node --test runtime_output_contract` remains green to guard entrypoint
  error/output behavior.

## Files To Touch

- `specs/6307-main-endpoint-orchestration-extraction.md`
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/runtime_entrypoint.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`

## Error Semantics

- Preserve current fail-closed error behavior:
  - endpoint serve failures remain mapped via `ConfigError::RuntimeDaemonLifecycle`.
  - observability endpoint still fails when runtime snapshot is unavailable.
- Preserve existing logging event names and field sets for endpoint start/complete events.

## Test Plan

- RED:
  - add extraction contract assertions that fail while logic remains inline.
- GREEN:
  - extract endpoint-serving helpers/orchestration into `runtime_entrypoint.rs`.
  - delegate from `run()`.
- REFACTOR:
  - keep module boundaries and naming explicit.
  - remove dead inline code in `main.rs`.
- Verification:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
  - `cargo test -p kamn-node --test runtime_output_contract`

## Phase 6 Integration Evidence (to fill at close)

- `cargo test -p kamn-node --test main_module_extraction_contract`
- `cargo test -p kamn-node --test runtime_output_contract`

Observed results:
- `crates/kamn-node/src/main.rs` reduced from 773 LOC to 689 LOC.
- `run()` delegates post-runtime endpoint orchestration through
  `runtime_entrypoint::serve_runtime_endpoints(...)`.
- `cargo test -p kamn-node --test main_module_extraction_contract` passed.
- `cargo test -p kamn-node --test runtime_output_contract` passed.
- `cargo test -p kamn-node` passed.
