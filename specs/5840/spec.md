# Spec: Issue #5840 - Harden cfg(test) Parsing in Production Expect Inventory

- Issue: #5840
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Production `expect()` inventory logic strips `#[cfg(test)]` items before counting, but the current skipper uses naive brace counting. Braces inside string literals and raw strings can desynchronize item skipping and leak test-only `expect()` calls into production counts.

## Scope
In scope:
- Harden `scripts/ci/check_no_production_expect.py` cfg(test)-item skipper to ignore braces inside string/comment contexts.
- Harden `crates/kamn-core/tests/review_r53_docs_contract.rs` inventory parser to the same semantics.
- Add regression fixtures for brace-heavy cfg(test) modules proving test-only `expect()` calls do not leak.
- Align inventory formula marker text with actual counting semantics.

Out of scope:
- Full Rust parser integration.
- Broad panic-path replacement across all crates.

## Acceptance Criteria
- AC-1: cfg(test) item stripping ignores braces in string literals, raw strings, and comments while scanning item boundaries.
- AC-2: Production `expect()` inventory in `review_r53_docs_contract.rs` uses the hardened cfg(test) semantics and remains deterministic.
- AC-3: Regression tests prove brace-heavy cfg(test) fixtures do not leak test-only `expect()` calls into production counts.
- AC-4: Regression tests prove production `expect()` calls following top-level cfg(test) imports are still counted.
- AC-5: Review marker formula strings describing production `expect()` inventory match implemented semantics.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | cfg(test) module with braces inside string literal | skipper exits at correct boundary |
| C-02 | AC-1 | Unit | cfg(test) module with raw string containing `{}` and `.expect(` | test-only expect excluded |
| C-03 | AC-2 | Functional | `review_r53_docs_contract` inventory run | deterministic count with no parser drift |
| C-04 | AC-3 | Regression | checker fixture with brace-heavy cfg(test) expect | checker remains pass |
| C-05 | AC-4 | Regression | top-level cfg(test) import + production expect | violation/count detected |
| C-06 | AC-5 | Functional | review marker formula checks | marker text equals parser semantics |

## Test Mapping
- `bash scripts/ci/test_check_no_production_expect.sh`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- No false positive inventory drift from cfg(test) brace-heavy fixtures.
- Docs-contract and checker fixtures agree on cfg(test) stripping behavior.
- Marker text no longer overclaims raw line counting behavior.
