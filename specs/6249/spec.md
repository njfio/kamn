# Issue 6249 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
Wave-1 extraction created dedicated crates, but `kamn-core` still carries multiple compatibility re-export shim modules. The remaining shim surface keeps crate boundaries blurry and increases maintenance burden.

## Scope
In scope:
- Inventory `kamn-core` compatibility re-export shims and assign keep/remove decisions.
- Migrate in-repo consumers to extracted crate imports for selected shim modules.
- Remove obsolete shims or mark explicitly time-bounded deprecation paths.

Out of scope:
- Breaking external API contracts without deprecation coverage.
- Full extraction of every remaining `kamn-core` subsystem.

## Acceptance Criteria
- AC-1: A shim inventory document exists with module-level decision (`remove`, `temporary-compat`, `retain`) and rationale.
- AC-2: At least three shim modules are migrated to direct extracted-crate usage and corresponding `kamn-core` shim-only wrappers are removed.
- AC-3: Any retained compatibility surface has explicit deprecation/removal plan and regression tests.
- AC-4: Targeted cross-crate tests verify no behavior regression after migration.

## Conformance Cases
- C-01 (AC-1, Conformance): Inventory includes current shim modules and decisions.
- C-02 (AC-2, Integration): Workspace references for migrated modules resolve through extracted crates rather than `kamn-core` shims.
- C-03 (AC-3, Regression): Retained compatibility modules include explicit timeline or follow-up issue link.
- C-04 (AC-4, Functional): Cross-crate tests for migrated paths pass and preserve behavior.

## Success Metrics
- Reduced `kamn-core` compatibility shim surface.
- Clear extraction boundary ownership and migration trajectory for remaining shims.
