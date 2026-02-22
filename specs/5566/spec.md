# Issue #5566 Spec - PRD Phase-4b Harness Run/Verify Command Contracts

- Status: Implemented
- Issue: #5566
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
`kamn-e2e-harness` currently supports only a minimal `--mode` argument and does not provide PRD-aligned `run` and `verify` command surfaces from sections 9, 11, and 12.

## Scope
In scope:
- Add deterministic command parser for:
  - `run --mode <mode> --evidence-dir <path> --scenarios <csv>`
  - `verify --evidence-dir <path> --kolme-chain-dump <path> --output <path>`
- Add scenario CSV parsing/validation against canonical `S-01..S-15` IDs.
- Integrate deterministic verification report JSON generation into `verify` execution path and output-file contract.
- Add RED->GREEN conformance tests for parser/selection/report contracts.
- Add phase-4b docs/research status markers.

Out of scope:
- CI workflow updates.
- Live infrastructure orchestration runtime execution.

## Acceptance Criteria
- AC-1: Harness supports explicit `run` and `verify` commands with required PRD-aligned flags.
- AC-2: Scenario CSV parsing validates IDs against full matrix and preserves deterministic order of selected IDs.
- AC-3: `run` command output includes selected mode/evidence-dir/scenario count markers.
- AC-4: `verify` command generates deterministic JSON report markers and writes output to requested path.
- AC-5: RED->GREEN conformance tests validate command parser, scenario selection, and verify output contracts.
- AC-6: phase-4b docs/research markers are present and coherent.
- AC-7: quality gates pass (`fmt`, `clippy`, targeted tests + regressions).

## Conformance Cases
- C-01 (AC-1): parser accepts valid `run` command with all required flags.
- C-02 (AC-1): parser accepts valid `verify` command with all required flags.
- C-03 (AC-1): parser rejects missing required flag values.
- C-04 (AC-2): scenario CSV selection accepts known IDs and preserves user-specified deterministic order.
- C-05 (AC-2): unknown scenario ID fails with explicit error.
- C-06 (AC-3): `run` execution output reports selected mode + selected scenario count.
- C-07 (AC-4): `verify` writes deterministic report JSON file containing `schema_check`, `proof_check`, `chain_check`, `content_check`.
- C-08 (AC-4): repeated verify execution with same manifest content yields byte-identical report output.
- C-09 (AC-5): RED failures observed before implementation and GREEN pass observed after implementation.
- C-10 (AC-6): phase-4b docs/research markers present and coherent.
- C-11 (AC-7): `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, and phase-1/2 regression suites pass.

## Success Metrics / Observable Signals
- Harness command contract surface matches PRD examples and can be invoked deterministically in CI lanes.
- Scenario selection and verify output behavior is deterministic and testable offline.
