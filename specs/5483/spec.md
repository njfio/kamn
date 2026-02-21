# Issue #5483 Spec - Daemon Topology Contract Test Decomposition

- Status: Accepted
- Issue: #5483
- Parent: #3812
- Milestone: R50.7 Daemon topology contract test decomposition

## Problem Statement
`crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests.rs` is a 2,400-line monolithic include file, which increases review friction, merge-conflict probability, and maintenance risk for ongoing topology contract hardening.

## Scope
In scope:
- Decompose the monolithic topology contract test file into cohesive include submodules.
- Preserve all existing test function names and behavior.
- Keep daemon test include ordering deterministic.

Out of scope:
- New topology contract logic or semantics.
- Shell/workflow/template surface changes.
- Renaming existing test functions.

## Acceptance Criteria
- AC-1: `live_postgres_topology_contract_tests.rs` is reduced to an include-hub that routes to new submodule files.
- AC-2: All existing topology contract test names remain unchanged and compile under the new submodule layout.
- AC-3: Targeted topology contract test runs remain green; fmt/clippy gates remain green.

## Conformance Cases
- C-01 (Structural, AC-1): Root topology test file contains include-based decomposition entries and no monolithic inline test body.
- C-02 (Functional/Regression, AC-2): Existing named topology tests execute without rename/path regressions.
- C-03 (Verification, AC-3): Targeted topology suite + fmt + strict clippy complete successfully.

## Success Metrics / Observable Signals
- Root topology test file shrinks from monolithic body to thin include hub.
- Topology test behavior remains unchanged across deterministic targeted runs.
- CI fast gate remains green for decomposition PR.
