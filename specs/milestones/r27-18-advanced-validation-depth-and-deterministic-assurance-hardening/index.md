# R27.18 Advanced validation depth and deterministic assurance hardening

## Milestone Summary

Validation-hardening tranche focused on deterministic assurance depth:
- property-based lifecycle invariants for task, escrow, and peer domains,
- deterministic fuzz governance for parser-risk surfaces,
- concurrency stress contract lanes with CI-local budget boundaries.

Milestone objective: increase defect-detection depth while keeping merge-gate costs bounded and fail-closed policy markers deterministic.

## Issue Hierarchy

- Epic:
  - `#4128` — Epic: R27.18 advanced validation depth and deterministic assurance hardening
- Stories:
  - `#4129` — Story: enforce property-based state-machine invariants across task escrow and peer lifecycle domains
  - `#4130` — Story: add fuzzing and concurrency stress governance with low-cost CI and local-heavy boundaries
- Tasks:
  - `#4131` — Task: add proptest invariants for task escrow and peer lifecycle transition correctness
  - `#4132` — Task: modularize high-density test surfaces for isolation and parallel execution
  - `#4133` — Task: add deterministic fuzz harness governance for message envelope and did parser surfaces
  - `#4134` — Task: add concurrency stress lanes and race-safety contract checks with ci-local budget controls
- Subtasks:
  - `#4135` — Subtask: add red proptest cases for transition legality and invariant preservation
  - `#4136` — Subtask: implement invariant helper library and deterministic seed configuration for property runners
  - `#4137` — Subtask: split monolithic test suites into domain modules with parity-preserving red tests
  - `#4138` — Subtask: add regression checks for test discovery stability and parallel execution boundaries
  - `#4139` — Subtask: add red fuzz-corpus drift tests and parser failure-taxonomy assertions
  - `#4140` — Subtask: implement bounded deterministic fuzz runner contracts with seed provenance markers
  - `#4141` — Subtask: add ci smoke checker for concurrency marker lineage and local-heavy lane exclusions
  - `#4142` — Subtask: update validation-depth docs and drift-contract checks for property fuzz concurrency closure

## Governance Markers

- `deterministic_property_seed_policy=required`
- `proptest_regression_corpus_tracking=required`
- `ci_smoke_local_heavy_boundary=required`
- `fuzz_and_concurrency_marker_taxonomy=fail_closed`
