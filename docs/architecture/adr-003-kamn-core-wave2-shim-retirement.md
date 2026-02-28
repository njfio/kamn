# ADR-003: kamn-core Wave2 Shim Retirement And Direct Extracted-Crate Wiring

## Context

ADR-002 extracted runtime guard contracts from `kamn-core` and retained module-level
compatibility shims to preserve public paths. With extraction stabilized, continuing to route
core exports through shim modules obscures ownership boundaries and slows follow-on decomposition.

Issue link: `#6249`
Spec link: `specs/6249/spec.md`

## Decision

For wave2 extraction completion:

- Keep compatibility shim modules in `kamn-core` only as temporary deprecated surfaces.
- Rewire `kamn-core` root exports for migrated domains to re-export directly from extracted crates
  (`kamn-runtime-guards`, `kamn-live-probe-matrix`, `kamn-bridges`) instead of via shim modules.
- Migrate in-repo consumers to import extracted crates directly for migrated domains.
- Set explicit shim retirement target milestone: **R61**.

## Consequences

- Ownership boundaries are now explicit at `kamn-core` root export level.
- In-repo consumers can converge on extracted crates without relying on compatibility modules.
- Deprecated shim modules remain available short-term for downstream compatibility, but each use
  now carries an explicit deprecation path and timeline.
