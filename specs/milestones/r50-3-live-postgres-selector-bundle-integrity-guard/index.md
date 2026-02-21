# R50.3 Milestone - Live-Postgres Selector Bundle Integrity Guard

## Context
R50.2 added selector-row CSV telemetry emission. This milestone hardens runtime behavior by validating selector bundle integrity invariants before daemon completion marker emission.

## Scope
- Add runtime selector bundle validation guard.
- Enforce unique rows, selector prefix compliance, and row-count parity.
- Add deterministic tests for pass/fail validation paths.

## Deliverables
- Issue #5475: runtime selector bundle integrity guard and tests.

## Exit Criteria
- Runtime guard executes in daemon path before completion marker emission.
- Invalid selector bundles fail with deterministic reason codes.
- Issue #5475 merged with spec marked Implemented.
