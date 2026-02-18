# Issue #3637 Spec

- Title: `Task: extract signer policy module for profile normalization and quorum checks`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
`crates/kamn-node/src/signer.rs` still aggregates multiple responsibilities, including policy checks, managed backend control, and nonce retry orchestration. This increases review risk and obscures ownership boundaries.

## Scope
In:
- Preserve `signer_policy` ownership for profile normalization, key-source resolution, and quorum linkage contracts.
- Extract managed backend control and nonce retry orchestration into dedicated signer submodules.
- Preserve deterministic reason-code behavior and existing public signer API boundaries.
- Keep parity with existing runtime commit signing behavior.

Out:
- Redesign of signing algorithms or provider protocols.
- Wire-format changes for runtime commit payloads.

## Acceptance Criteria
- AC-1: Given signer module wiring, when inspecting `signer.rs`, then managed backend control and nonce retry logic are delegated to dedicated signer submodules.
- AC-2: Given managed-external signing flow, when backend command/provenance validation runs, then deterministic fail-closed reason codes are preserved.
- AC-3: Given nonce fetch failures, when retry policy evaluates timeout/unavailable/malformed responses, then retry/backoff behavior remains deterministic and bounded.
- AC-4: Given existing signer tests, when running scoped kamn-node signer suites, then unit/functional/integration/regression coverage remains passing.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration/Conformance | `rg -n "mod managed_backend|mod nonce" crates/kamn-node/src/signer.rs` | signer module exposes dedicated managed-backend and nonce modules |
| C-02 | AC-2 | Functional/Regression | `cargo test -p kamn-node signer -- --nocapture` | managed backend response/provenance tests pass with deterministic reason codes |
| C-03 | AC-3 | Unit/Functional | `cargo test -p kamn-node signer -- --nocapture` | nonce retry classifier/backoff tests remain deterministic and bounded |
| C-04 | AC-4 | Integration/Regression | `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | runtime signer integration behavior remains parity-stable |

## Test Mapping
- C-01: module boundary assertion via source inspection and compilation.
- C-02: `crates/kamn-node/src/main_tests/signer_tests.rs` managed-external signer tests.
- C-03: `crates/kamn-node/src/signer.rs` (or signer submodule) nonce retry unit tests.
- C-04: `crates/kamn-node/src/main_tests/signer_tests.rs` runtime signing flows.

## Success Metrics
- `signer.rs` line count decreases and module ownership is explicit.
- Scoped signer tests stay green without behavior drift.
- Managed backend and nonce retry reason taxonomy remains deterministic.
