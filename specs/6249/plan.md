# Issue 6249 Plan

## Approach
1. Enumerate current compatibility re-export modules in `kamn-core`.
2. Select wave-2 migration subset based on risk and consumer readiness.
3. Add RED tests asserting direct extracted-crate usage for selected modules.
4. Migrate imports/usages and remove obsolete shims.
5. Add deprecation/timeline markers for any retained shim and verify with regression tests.

## Affected Modules
- `crates/kamn-core/src/*` shim modules (selected subset)
- Consumer crates importing shim surfaces (selected subset)
- `crates/kamn-core/tests/*` extraction compatibility tests
- `docs/architecture/` extraction boundary documentation
- `docs/planning/r59-followup.md`
- `specs/6249/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: removing shims causes broad compile breakage.
  - Mitigation: migrate in small slices with focused cross-crate checks.
- Risk: hidden consumer reliance on old paths.
  - Mitigation: search-based inventory and targeted regression tests for each migrated module.
- Risk: retained shims become permanent.
  - Mitigation: require explicit deprecation/removal timeline and follow-up IDs.

## Interfaces
- Internal crate import boundaries and compatibility façades.
- No wire-format/protocol changes.
