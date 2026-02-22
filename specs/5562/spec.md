# Issue #5562 Spec - PRD Phase-3 kamn-e2e-harness Scaffold and Core Scenario Contracts

- Status: Reviewed
- Issue: #5562
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
PRD phase-3 requires `kamn-e2e-harness` for mode-driven scenario orchestration and evidence/verification handling, but the crate and required module layout are missing.

## Scope
In scope:
- Add workspace crate `crates/kamn-e2e-harness` with PRD section-13 structure.
- Implement deterministic execution-mode and scenario registries for core scenarios (`S-01`, `S-02`, `S-03`, `S-04`, `S-05`, `S-06`, `S-08`).
- Implement evidence manifest model + offline verification scaffold.
- Add conformance tests for required paths, mode registry, scenario inventory, and manifest schema markers.
- Add phase-3 docs/research status markers.

Out of scope:
- CI workflow changes.
- Full live infrastructure orchestration against external runtimes in this slice.

## Acceptance Criteria
- AC-1: `crates/kamn-e2e-harness` required files exist and compile.
- AC-2: Harness mode registry exposes `sdk-direct`, `cli-scripted`, `mcp-tau`, `mcp-any`.
- AC-3: Scenario registry contains core PRD scenarios (`S-01`, `S-02`, `S-03`, `S-04`, `S-05`, `S-06`, `S-08`).
- AC-4: Evidence manifest schema marker and verifier scaffold align with PRD section 8/9 expectations.
- AC-5: RED->GREEN conformance tests validate structure/mode/scenario/manifest contracts.
- AC-6: Phase-3 docs/research status markers are present.
- AC-7: Quality gates pass (`cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, targeted tests).

## Conformance Cases
- C-01 (AC-1): all required harness paths exist.
- C-02 (AC-1): workspace compiles `kamn-e2e-harness`.
- C-03 (AC-2): mode registry contains exactly 4 required modes.
- C-04 (AC-3): scenario registry contains required core scenario IDs.
- C-05 (AC-4): evidence manifest schema version marker is deterministic.
- C-06 (AC-4): verifier accepts deterministic manifest baseline.
- C-07 (AC-5): RED failures observed and GREEN pass recorded.
- C-08 (AC-6): phase-3 docs/research markers present and coherent.
- C-09 (AC-7): fmt/clippy/tests green.

## Success Metrics / Observable Signals
- `kamn-e2e-harness` crate compiles and passes conformance tests.
- Mode/scenario/manifest registries are deterministic and consumable by later phase-4 integration.
- PRD delivery advances from wrappers to orchestrator foundation.
