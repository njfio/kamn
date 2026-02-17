# Spec — Issue #4821

- Title: Subtask: add wrapper_name/phase manifest fields and rewrite non-kolme dispatcher resolution
- Parent: Parent task: #4812
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Replace hardcoded non-Kolme wrapper/phase case statements with manifest-declared `wrapper_name`/`phase` metadata and deterministic resolver behavior.

## Problem Statement

Without manifest-declared wrapper/phase metadata, the dispatcher remains a large hardcoded shell mapping hotspot and cannot scale safely.

## Scope

In scope:
- add manifest metadata fields for non-Kolme wrapper/phase routing
- rewrite dispatcher manifest/phase resolution to consume manifest metadata
- add/update deterministic tests for metadata-backed dispatch behavior
- spec/docs updates for changed behavior

Out of scope:
- phase work outside this subtask boundary
- unrelated refactors

## Acceptance Criteria

- AC-1: Non-Kolme dispatch resolution is manifest-backed (wrapper and phase resolved from manifest metadata).
- AC-2: Dispatcher compatibility is preserved across existing non-Kolme wrapper matrix suites.
- AC-3: Red/green evidence is captured for manifest metadata enforcement.

## Conformance Cases

- C-01 (AC-1/AC-3): `bash scripts/framework/test_non_kolme_manifest_backed_contract_lane_dispatch_wrapper_matrix.sh` fails before migration (missing `wrapper_name`), then passes after metadata migration.
- C-02 (AC-2): full non-Kolme dispatcher matrix suite passes (`scripts/framework/test_non_kolme*contract_lane_dispatch_wrapper_matrix.sh`).
- C-03 (AC-2): `bash scripts/bridge/test_bridge_deep_lane_dispatch_wrapper_matrix.sh` passes to validate deep/run phase behavior under metadata-backed resolution.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Dispatcher shell mapping complexity is reduced by removing hardcoded wrapper/phase case blocks.
