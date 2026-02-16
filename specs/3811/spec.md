# Issue #3811 Spec

- Title: `Subtask: enforce signer_adapter API boundary and re-export drift contracts`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer adapter extraction can regress when crypto/key-source logic drifts back into `signer.rs` or when re-export boundaries change without detection.

## Scope
In:
- Extract adapter-owned primitives into `crates/kamn-node/src/signer/signer_adapter.rs`.
- Re-export adapter-owned symbols from `crates/kamn-node/src/signer.rs`.
- Add source/docs contract tests that fail closed on module ownership drift.
- Update architecture docs with adapter boundary markers and guard command.

Out:
- New cryptographic algorithms.
- Signer policy/quorum contract changes.

## Acceptance Criteria
- AC-1: Given signer module boundaries, when adapter ownership is evaluated, then `signer_adapter` owns signing/key-source primitives and `signer.rs` re-exports the adapter API.
- AC-2: Given ownership drift, when adapter logic is re-inlined into `signer.rs` or re-exports drift, then contract tests fail closed.
- AC-3: Given docs parity checks, when adapter boundary markers/guard command drift, then docs-contract tests fail closed.
- AC-4: Given scoped signer suites, when adapter boundary extraction is applied, then signer behavior remains parity-stable.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | `cargo test -p kamn-node --test signer_adapter_boundary_contract source_declares_signer_adapter_boundary_re_exports -- --exact --nocapture` | signer source declares adapter module + re-exports |
| C-02 | AC-2 | Regression/Conformance | `cargo test -p kamn-node --test signer_adapter_boundary_contract source_enforces_signer_adapter_ownership_without_reinline_backslide -- --exact --nocapture` | backslide/re-export drift fails closed |
| C-03 | AC-3 | Docs/Conformance | `cargo test -p kamn-node --test signer_adapter_boundary_contract docs_declare_signer_adapter_boundary_markers -- --exact --nocapture` | docs marker/guard parity remains enforced |
| C-04 | AC-4 | Integration/Regression | `cargo test -p kamn-node signer -- --nocapture` + `cargo test -p kamn-node main_tests::signer_tests -- --nocapture` | signer behavior remains parity-stable |

## Test Mapping
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-node/src/signer.rs`

## Success Metrics
- signer adapter module ownership is explicit, re-export drift is fail-closed, and signer regression suites stay green.
