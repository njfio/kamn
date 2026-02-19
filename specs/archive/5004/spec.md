# Issue #5004 Spec

- Title: Story: M1 trust anchor merkle batching and Kolme anchoring integration
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Deliver PRD M1 trust-anchor integration for deterministic merkle batching,
inclusion-proof verification, Kolme anchoring worker behavior, and anchoring
failure-matrix drift evidence. Story delivery is completed through child issues
`#5017` (M1 contracts) and `#5030` (proof/matrix conformance expansion).

## Acceptance Criteria
- AC-1: M1 trust-anchor contracts are implemented for deterministic merkle batch
  assembly, inclusion-proof generation/verification, and idempotent Kolme
  anchoring outcomes.
- AC-2: Deterministic proof decision and anchoring failure-matrix coverage
  exists with stable reason markers and fail-closed invalid-input handling.
- AC-3: Story maps to PRD M1 requirements and critical test scenarios with
  reproducible evidence.
- AC-4: Shell/workflow/python/template LOC remains unchanged across story
  implementation (`shell_loc_delta_actual = 0` aggregate for child issues).

## Scope
In scope:
- Story-level completion evidence for child deliverables `#5017` and `#5030`.
- Story artifact normalization (`spec.md`, `plan.md`, `tasks.md`) to implemented
  lifecycle status.
- PRD mapping and deterministic test-evidence traceability for M1.

Out of scope:
- New dependencies/protocol/wire-format changes.
- Additional M1 feature expansion beyond accepted child issue scopes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run M1 contract suite (`data_layer_m1_merkle_anchoring`) | Merkle batching/proof verification/Kolme anchoring flows pass deterministically |
| C-02 | AC-2 | Conformance | Run M1 proof decision + failure-matrix cases (`spec_c06..spec_c09`) | Stable/drift decisions and fail-closed invalid-input behavior pass with stable reason markers |
| C-03 | AC-3 | Regression | Run crate-level regression and verify PRD M1 scenario mapping | M1 contracts remain green with deterministic outputs |
| C-04 | AC-4 | Regression | Story child-diff shell/rust audit + guardrail evidence | `shell_loc_delta_actual = 0`; shell ratio posture improved |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m1_merkle_anchoring`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5004.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5004.json`

## Success Metrics
- Story `#5004` closes with both child issues merged (`#5017`, `#5030`) and
  ACs mapped to passing deterministic tests.
- M1 proof and anchoring matrix contract suites remain passing.
- Shell-to-Rust guardrails remain in-go with zero shell delta for story work.
