# Issue 6275 - Establish enforced kamn-core decomposition map and tranche ordering

## Objective
Create a fail-closed, discoverable decomposition map for `kamn-core` that defines tranche ordering, module-group boundaries, and extraction destinations so decomposition work is executable and auditable.

## Inputs/Outputs
- Inputs:
  - Existing architecture map: `docs/architecture/kamn-core-module-map.md`
  - Architecture index: `docs/architecture/README.md`
  - Current `kamn-core` source footprint and largest-file hotspots under `crates/kamn-core/src`
- Outputs:
  - New decomposition-map section in `docs/architecture/kamn-core-module-map.md` with deterministic markers and tranche ordering.
  - Contract tests that fail closed when required decomposition sections/markers drift.
  - Architecture index link to decomposition-map anchor for discoverability.

## Boundaries/Non-goals
- In scope:
  - Documentation and doc-contract tests for decomposition map governance.
  - Largest-file contributor hotspot listing for extraction prioritization.
- Out of scope:
  - Moving modules between crates.
  - Runtime behavior changes.
  - Public API changes.
  - Adding dependencies.

## Failure modes
- FM1: Decomposition map exists but omits explicit tranche ordering or extraction destinations.
- FM2: Decomposition-map markers/sections drift silently because tests do not enforce them.
- FM3: Top monolith hotspots are missing, making extraction prioritization ungrounded.
- FM4: Map is not linked from architecture navigation and is difficult to discover.

## Acceptance criteria (testable booleans)
- AC1: `docs/architecture/kamn-core-module-map.md` contains a decomposition-map section with explicit tranche ordering and target extraction destinations.
- AC2: `crates/kamn-core/tests/kamn_core_decomposition_map_docs.rs` enforces deterministic marker/section presence for the decomposition map.
- AC3: Decomposition map includes the current top `kamn-core` monolith hotspot files by LOC (including `message_lifecycle.rs`, `channel_models.rs`, `p2p_transport_live.rs`, `task_operations.rs`, and `did_registry.rs`).
- AC4: `docs/architecture/README.md` links to the decomposition-map anchor.

## Files to touch
- `docs/architecture/kamn-core-module-map.md`
- `docs/architecture/README.md`
- `crates/kamn-core/tests/kamn_core_decomposition_map_docs.rs`

## Error semantics
- Documentation drift is treated as a hard test failure in doc-contract suites.
- Missing required markers/anchors is fail-closed (no silent fallback assumptions).

## Test plan
- RED:
  - Add new doc-contract tests for decomposition map markers/sections and architecture index link.
  - Run targeted test and confirm failure before docs updates.
- GREEN:
  - Add decomposition map + hotspot table + markers and architecture index link.
  - Re-run targeted test to green.
- REFACTOR:
  - Keep test helpers small and explicit; avoid duplicated marker parsing logic.
- INTEGRATION:
  - Run targeted docs-contract suites that cover architecture map discoverability and decomposition markers.
