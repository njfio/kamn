# Issue #3913 Spec

- Title: `Subtask: add explicit zeroization to signer key decode and transient buffers`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Decoded/intermediate key buffers must be scrubbed across success and failure paths to reduce key-material exposure risk.

## Scope
In:
- Preserve explicit zeroization markers for decoded bytes and key hex buffers.
- Guard decode path behavior with deterministic regression checks.

Out:
- Policy checker wiring.

## Acceptance Criteria
- AC-1: Decode failure paths scrub transient key buffers before returning.
- AC-2: Decode success paths scrub transient key buffers after signer construction.
- AC-3: Contract tests fail closed if zeroization markers drift from source/docs.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Regression | `cargo test -p kamn-node signer::tests::regression_signer_private_key_parse_zeroizes_hex_buffer_on_failure -- --exact --nocapture` | failure path scrub remains enforced |
| C-02 | AC-2 | Unit/Regression | `cargo test -p kamn-node signer::tests::unit_signer_private_key_parse_zeroizes_hex_buffer_on_success -- --exact --nocapture` | success path scrub remains enforced |
| C-03 | AC-3 | Conformance | `cargo test -p kamn-node --test signer_secret_hygiene_contract source_declares_signer_decode_zeroization_markers -- --exact --nocapture` | source/docs zeroization markers remain present |

## Test Mapping
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/tests/signer_secret_hygiene_contract.rs`

## Success Metrics
- explicit signer decode zeroization guarantees stay regression-locked.
