# Issue #5003 Spec

- Title: Story: M0 foundation schema, append-only enforcement, and envelope crypto pipeline
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M0 foundation contracts for deterministic envelope record
derivation, append-only controls, compression validation, and hash-chain tamper
detection while preserving shell-surface neutrality. Story delivery is completed
through child issues `#5016` (foundation contracts) and `#5029` (deterministic
conformance matrix coverage).

## Acceptance Criteria
- AC-1: M0 foundation contracts are implemented and integrated in `kamn-core`
  for envelope-crypto determinism, append-only duplicate rejection, compression
  constraints, and hash-chain verification.
- AC-2: Deterministic conformance matrix coverage exists for M0 invariants with
  stable `Stable`/`DriftDetected` reason markers and fail-closed invalid-input
  behavior.
- AC-3: Story maps to PRD M0 requirements and critical scenario coverage with
  reproducible test evidence.
- AC-4: Shell/workflow/python/template LOC remains unchanged across story
  implementation (`shell_loc_delta_actual = 0` aggregate for child issues).

## Scope
In scope:
- Story-level completion evidence for child deliverables `#5016` and `#5029`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented
  lifecycle status.
- PRD mapping and deterministic test-evidence traceability for M0.

Out of scope:
- New dependencies/protocol/wire-format changes.
- Additional M0 feature expansion beyond accepted child issue scopes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run M0 contract suite (`data_layer_m0_contract`) | Envelope determinism, append-only duplicate rejection, compression validation, and hash-chain verification pass |
| C-02 | AC-2 | Conformance | Run M0 conformance matrix cases (`spec_c05..spec_c07`) | Stable/drift decisions and fail-closed invalid-input behavior pass with stable reason markers |
| C-03 | AC-3 | Regression | Run crate-level regression and verify PRD scenario mapping | M0 contracts remain green with deterministic outputs |
| C-04 | AC-4 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; shell ratio posture improved |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m0_contract`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5003.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5003.json`

## Success Metrics
- Story `#5003` closes with both child issues merged (`#5016`, `#5029`) and ACs
  mapped to passing deterministic tests.
- M0 conformance matrix and foundation contract suites remain passing.
- Shell-to-Rust guardrails remain in-go with zero shell delta for story work.
