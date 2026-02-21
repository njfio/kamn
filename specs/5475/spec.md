# Issue #5475 Spec - Runtime Selector Bundle Integrity Guard

- Status: Implemented
- Issue: #5475
- Parent: #3812
- Milestone: R50.3 Live-postgres selector bundle integrity guard

## Problem Statement
Daemon runtime emits live-postgres selector bundle telemetry but does not enforce runtime integrity invariants on selector bundle content, allowing potential marker incoherence if selector data becomes malformed.

## Scope
In scope:
- Add runtime selector bundle validation helper with deterministic reason codes.
- Integrate guard in daemon execution before completion marker emission.
- Add tests covering valid bundle and invalid synthetic bundles.

Out of scope:
- New telemetry schema keys.
- Additional topology/multi-host execution lanes.

## Acceptance Criteria
- AC-1: Runtime selector bundle validation enforces unique rows and selector prefix contract.
- AC-2: Runtime selector bundle validation enforces row-count parity with configured selector bundle row count.
- AC-3: Daemon execution path invokes selector bundle guard and tests validate deterministic pass/fail behavior.

## Conformance Cases
- C-01 (Unit, AC-1): validation helper returns `Ok` for canonical runtime selector rows and deterministic error for duplicate/malformed prefix rows.
- C-02 (Unit, AC-2): validation helper returns deterministic error when expected row count mismatches selector bundle length.
- C-03 (Functional/Conformance, AC-3): daemon phase6 applied marker contract test remains green with guard in path.

## Success Metrics / Observable Signals
- Runtime selector telemetry cannot be emitted from malformed bundles.
- Failure reason codes are deterministic and testable.
- Existing runtime marker tests continue to pass.
