# Spec: Issue 6333 - Extract service API model structs into dedicated module

## Objective
Extract service API response/event model structs from `crates/kamn-sdk/src/service.rs` into a dedicated module, keeping `service.rs` focused on client orchestration and transport/parsing flow.

## Inputs/Outputs
- Inputs:
  - Inline public model structs in `service.rs`:
    - `ServiceMessageReceipt`
    - `ServiceMessageStatus`
    - `ServiceChannelReceipt`
    - `ServiceChannelMessages`
    - `ServiceTaskReceipt`
    - `ServiceTaskStatus`
    - `ServiceEscrowStatus`
    - `ServiceContentRegistration`
    - `ServiceContentStatus`
    - `ServiceBridgeSubmission`
    - `ServiceBridgeStatus`
    - `ServiceAgentProfile`
    - `ServiceHealthStatus`
    - `ServiceRouteEvent`
- Outputs:
  - New `service_models.rs` module with model struct definitions.
  - `service.rs` module wiring + re-exports preserving public type surface.
  - Extraction contract assertions for module wiring and inline-removal.

## Boundaries/Non-goals
- Do not change model field names/types/order.
- Do not change parse/request behavior.
- Do not add dependencies.

## Failure modes
- Missing re-exports break consumer imports.
- Any field/type drift causes compile/runtime regressions.
- Missing module wiring causes contract failures.

## Acceptance criteria (testable booleans)
- [ ] `service.rs` declares `mod service_models;`.
- [ ] `service.rs` no longer contains inline definitions for extracted model structs.
- [ ] Existing public model names and fields remain unchanged.
- [ ] `service_module_extraction_contract` includes and passes model extraction assertions.
- [ ] Existing `kamn-sdk` tests covering service API behaviors remain green.

## Files to touch
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/service_models.rs` (new)
- `crates/kamn-sdk/tests/service_module_extraction_contract.rs`
- `specs/6333-service-models-module-extraction.md`

## Error semantics
- No error semantic changes (model extraction only).

## Test plan
- Red phase:
  - Extend `service_module_extraction_contract` with module/inline assertions for model extraction and confirm failure.
- Green/refactor/integration phases:
  - `cargo test -p kamn-sdk --test service_module_extraction_contract`
  - `cargo test -p kamn-sdk --lib`
  - `cargo test -p kamn-sdk --test service_api_client`

## Phase 6 integration evidence
- Pending implementation.

## Deviations
- None.
