# Issue #5608 Spec - R51 Milestone Closeout Marker Finalization

- Status: Implemented
- Issue: #5608
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
After merging #5607, the milestone index still advertises `#5606` as active instead of final completed state, leaving closeout markers inconsistent with issue state.

## Scope
In scope:
- Update milestone index completed/active markers for post-#5607 state.
- Mark delivery slice 25 as completed.
- Add docs-contract RED->GREEN test that enforces closeout markers.

Out of scope:
- Runtime behavior changes.
- New feature contracts.

## Acceptance Criteria
- AC-1: milestone index includes `#5606` in completed issue list.
- AC-2: milestone index marks active issue as `None`.
- AC-3: delivery slice 25 is marked completed.
- AC-4: RED->GREEN docs-contract test validates closeout markers.
- AC-5: touched crate tests pass.

## Conformance Cases
- C-01 (AC-1): index contains `#5606` under completed markers.
- C-02 (AC-2): index contains `Active issue(s): None`.
- C-03 (AC-3): delivery slice 25 line contains `(Completed)`.
- C-04 (AC-4): docs-contract test fails before marker updates.
- C-05 (AC-4): docs-contract test passes after marker updates.
- C-06 (AC-5): `cargo test -p kamn-e2e-harness --test phase6_runtime_validation_docs_contract` passes.

## Success Metrics / Observable Signals
- Milestone index no longer points to an active issue once all delivery slices are complete.
- Contract test protects closeout marker integrity.
