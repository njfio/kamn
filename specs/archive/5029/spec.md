# Issue #5029 Spec

- Title: Subtask: M0 conformance matrix for envelope crypto, append-only, and hash-chain invariants
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Parent task `#5016` delivered deterministic M0 envelope record derivation,
append-only duplicate protection, and hash-chain tamper detection. The
remaining high-risk gap is an explicit deterministic conformance-matrix
contract that reports stable/drift outcomes across those three invariants with
typed fail-closed input validation.

## Acceptance Criteria
- AC-1: M0 exposes a deterministic conformance-matrix API with stable
  `Stable`/`DriftDetected` decision reason markers for envelope-crypto
  determinism, append-only duplicate rejection, and hash-chain tamper-detection
  invariants.
- AC-2: Matrix evidence is deterministic and includes per-case mismatch markers
  plus invariant classification for every case.
- AC-3: Matrix evaluation fails closed for invalid case sets (empty cases or
  empty case id).
- AC-4: Existing M0 record derivation, append-only controls, and hash-chain
  behavior remain deterministic and passing.
- AC-5: Shell/workflow/python/template LOC remains unchanged
  (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- Add additive M0 conformance-matrix contracts/API in `data_layer_m0`.
- Export stable matrix decision reason constants and matrix types.
- Add conformance tests for stable/drift matrix evaluation and invalid-input
  fail-closed behavior.

Out of scope:
- New dependencies/protocol/wire-format changes.
- Shell/workflow/template/CI modifications.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Matrix cases for envelope-crypto deterministic hash outputs, append-only duplicate rejection, and hash-chain tamper detection | Decision is `Stable` with stable reason marker |
| C-02 | AC-2 | Conformance | Matrix including at least one intentionally mismatched invariant result | Decision is `DriftDetected` and mismatched case evidence is marked deterministically |
| C-03 | AC-3 | Regression | Empty matrix or case with empty `case_id` | Fail-closed typed error |
| C-04 | AC-4 | Regression | Existing `spec_c01`..`spec_c04` M0 tests | Existing deterministic behavior remains green |
| C-05 | AC-5 | Regression | Shell/rust guardrail checks + diff audit | No shell surface growth; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m0_contract`
- `cargo test -p kamn-core`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5029.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5029.json`

## Success Metrics
- Conformance-matrix API emits deterministic decisions and mismatch evidence for
  M0 invariant coverage.
- All `#5029` conformance cases pass in `data_layer_m0_contract`.
- Shell-to-Rust ratio and hard-ceiling guardrails remain in-go.
