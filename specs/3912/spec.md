# Issue #3912 Spec

- Title: `Task: zeroize signer key decode/loading intermediates across runtime profiles`
- Status: `Reviewed`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer key decode/loading paths must guarantee deterministic scrubbing of transient key buffers and protect error surfaces from raw key-material leakage.

## Scope
In:
- Verify decode/loading intermediates are zeroized in success/failure paths.
- Add regression checks for decode-failure hygiene and redaction safety.
- Update architecture docs with explicit decode-path zeroization guarantees.

Out:
- External custody redesign.
- Transport/runtime protocol changes.

## Acceptance Criteria
- AC-1: Given signer decode/loading paths, when key parsing succeeds/fails, then transient buffers are scrubbed deterministically.
- AC-2: Given decode failures, when errors are surfaced, then raw private-key values are not emitted.
- AC-3: Given docs and contract tests, when drift checks run, then zeroization guarantees remain documented and enforced.
- AC-4: Given scoped signer suites, when regression gates run, then signer behavior remains parity-stable.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Regression | `cargo test -p kamn-node signer::tests::regression_signer_private_key_parse_zeroizes_hex_buffer_on_failure -- --exact --nocapture` | failure path scrubs private-key hex buffer |
| C-02 | AC-2 | Functional/Regression | `cargo test -p kamn-node signer::tests::regression_signer_private_key_decode_failure_redacts_sensitive_input -- --exact --nocapture` | decode failure error does not contain raw key input |
| C-03 | AC-3 | Conformance/Docs | `cargo test -p kamn-node --test signer_secret_hygiene_contract -- --nocapture` | source/docs zeroization markers remain present |
| C-04 | AC-4 | Integration/Regression | `cargo test -p kamn-node signer -- --nocapture` | signer behavior remains parity-stable |

## Test Mapping
- C-01/C-02: `crates/kamn-node/src/signer.rs`
- C-03: `crates/kamn-node/tests/signer_secret_hygiene_contract.rs`
- C-04: `crates/kamn-node/src/main_tests/signer_tests.rs` and signer module tests

## Success Metrics
- Zeroization and redaction guarantees are enforced by deterministic tests.
- Architecture docs explicitly declare decode-path zeroization guarantees.
