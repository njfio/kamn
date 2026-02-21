# Issue #5499 Spec - Cross-Store Replay Policy Marker Helper API

- Status: Implemented
- Issue: #5499
- Parent: #3812
- Milestone: R50.15 Cross-store replay policy marker API

## Problem Statement
Callers currently compare `CrossStoreReplayConsistencyStatus` manually to decide policy marker text. This duplicates logic and risks drift from canonical consistency semantics.

## Scope
In scope:
- Add additive status helper API that maps `Consistent`/`Divergent` to deterministic policy markers.
- Add report-level helper exposing policy marker directly.
- Update contract lane binary to consume helper API.
- Add unit tests for helper behavior.

Out of scope:
- Reason taxonomy/schema changes.
- Dependency changes.

## Acceptance Criteria
- AC-1: `CrossStoreReplayConsistencyStatus` exposes deterministic policy marker mapping (`verified`/`violated`).
- AC-2: `CrossStoreReplayConsistencyReport` exposes a helper returning the same marker.
- AC-3: Contract lane binary uses the helper API instead of manual status comparison for policy marker output.
- AC-4: Unit tests cover status/report helper behavior and pass.

## Conformance Cases
- C-01 (AC-1): status helper returns `verified` for `Consistent`.
- C-02 (AC-1): status helper returns `violated` for `Divergent`.
- C-03 (AC-2): report helper delegates to status helper and returns expected marker in consistent and divergent paths.
- C-04 (AC-3): contract lane prints helper-derived policy marker and fails if marker is not `verified`.
- C-05 (AC-4): targeted tests pass in `kamn-core`.

## Success Metrics / Observable Signals
- Deterministic policy-marker logic is centralized in production API.
- Contract lane and tests consume the centralized helper.
