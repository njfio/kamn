# Issue #5030 Spec

- Title: Subtask: M1 deterministic merkle proof and Kolme anchoring failure-matrix coverage
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5017` delivered baseline deterministic merkle batching, inclusion
proof verification, and Kolme anchoring worker behavior. The remaining M1 gap
is explicit deterministic decision/report contracts for proof-verification
reason markers and anchoring failure-matrix drift evidence.

## Acceptance Criteria
- AC-1: M1 exposes a deterministic proof-verification decision wrapper that
  emits stable valid/invalid reason constants without replacing fail-closed
  verification semantics.
- AC-2: M1 exposes deterministic Kolme anchoring failure-matrix evaluation API
  with stable `Stable`/`DriftDetected` reason markers and per-case mismatch
  evidence over retry-class + outcome-kind expectations.
- AC-3: Failure-matrix evaluation fails closed for invalid case sets (empty
  matrix or empty case id).
- AC-4: Existing M1 merkle batching, inclusion-proof verification, and anchoring
  worker contracts remain deterministic and passing.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add additive M1 proof-decision and anchoring failure-matrix contracts in
  `data_layer_m1`.
- Export stable M1 reason constants for proof-decision and matrix decisions.
- Add conformance tests for stable/drift matrix outcomes and invalid-input
  fail-closed behavior.

Out of scope:
- New dependencies/protocol/wire-format changes.
- Shell/workflow/template/CI modifications.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Valid and tampered inclusion proofs | Decision wrapper returns stable valid/invalid reason constants |
| C-02 | AC-2 | Conformance | Anchoring results matrix with expected retry/outcome classes | Decision is `Stable` and evidence shows no mismatches |
| C-03 | AC-2 | Regression | Matrix with intentionally wrong expectation | Decision is `DriftDetected` with mismatch evidence |
| C-04 | AC-3 | Regression | Empty matrix or empty case id | Fail-closed typed errors |
| C-05 | AC-4 | Regression | Existing M1 `spec_c01`..`spec_c05` flows | Existing deterministic behavior remains green |
| C-06 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m1_merkle_anchoring`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5030.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5030.json`

## Success Metrics
- M1 proof and anchoring matrix decision contracts emit deterministic reason
  markers and mismatch evidence.
- All `#5030` conformance cases pass in `data_layer_m1_merkle_anchoring`.
- Shell-to-Rust ratio and hard-ceiling guardrails remain in-go.
