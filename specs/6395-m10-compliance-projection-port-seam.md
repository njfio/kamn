# Spec: Issue #6395 - M10 compliance projection port seam

## Objective

Unblock the remaining M10 extraction by introducing a data-layer-owned compliance projection port seam and routing core M10 shred-completeness projection logic through a compatibility adapter, while preserving existing `kamn-core` APIs.

## Inputs/Outputs

- Inputs:
  - `kamn-core` M10 projection path currently tied to `DataLayerM8ComplianceRegistry` and `KamnDid`-based owner normalization.
  - existing extraction map markers in `docs/architecture/data-layer-runtime-wiring.md`.
- Outputs:
  - new `kamn-data-layer` M10 compliance projection port trait + message model + typed errors.
  - new `kamn-core` M8-backed adapter implementing that port.
  - additive `kamn-core` projection entrypoint using the generic port seam.
  - docs/test markers proving blocker mitigation for projection path.

## Boundaries/Non-goals

- In scope:
  - M10 projection seam for `project_partition_shred_completeness*` path.
  - compatibility retention for existing `project_partition_shred_completeness_from_m8` entrypoint.
  - deterministic error mapping from port errors to existing M10 reason-code taxonomy.
- Out of scope:
  - full M10 module migration.
  - replacing M8 registry internals.
  - changing phase-6 orchestration APIs in this issue.

## Failure modes

- FM-1: seam trait is not exported from `kamn-data-layer`.
- FM-2: core projection path bypasses seam and remains directly coupled.
- FM-3: compatibility API regresses behavior/error reason codes.
- FM-4: docs markers fail to reflect seam mitigation status.
- FM-5: telemetry evidence missing for pre/post lane timing.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `kamn-data-layer` exports `DataLayerM10ComplianceProjectionPort` and supporting seam models/errors.
- [ ] AC-2: `kamn-core` adds `project_partition_shred_completeness_with_port(...)` and routes `project_partition_shred_completeness_from_m8(...)` through an M8 adapter.
- [ ] AC-3: Existing M10 core contract tests stay green (behavior compatibility preserved).
- [ ] AC-4: Docs contract enforces seam extraction markers and mitigation status.
- [ ] AC-5: Pre/post telemetry captured for `cargo test -p kamn-core --test data_layer_m10_partition_archival`.

## Files to touch

- `specs/6395-m10-compliance-projection-port-seam.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m10_compliance_projection_port.rs` (new)
- `crates/kamn-data-layer/tests/data_layer_m10_compliance_projection_port_contract.rs` (new)
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Port errors must remain typed and fail-closed:
  - owner-scope violation
  - lookup failure
  - invalid input
- Core adapter maps these to existing M10 projection reason markers:
  - `DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE`
  - `DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE`
  - `DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE`
- No silent fallbacks.

## Test plan

- RED:
  - extend docs contract with seam marker assertions.
  - add `kamn-data-layer` seam contract test that fails until the new port surface exists.
  - add core M10 test for the new `with_port` entrypoint using a fake port.
- GREEN:
  - implement `kamn-data-layer` seam module and exports.
  - add core M8 adapter + new generic projection entrypoint.
  - route existing `from_m8` method through adapter.
  - update docs markers.
- REFACTOR:
  - remove duplicated projection logic between old/new entrypoints.
  - keep adapter and mapper helpers focused and deterministic.
- INTEGRATION:
  - run docs contract, data-layer seam contract, full core M10 contract lane.
  - capture pre/post telemetry.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
