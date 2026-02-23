# Spec: Issue #5840 - Fix cfg(test) Parsing Drift in Production `expect()` Inventory

- Issue: #5840
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Production `expect()` inventory and panic-path checker outputs are inconsistent. The current
`#[cfg(test)]` item skipping logic uses line-level brace counting that can be confused by braces
inside string literals/raw strings and comments. This can leak test-only `expect()` callsites into
"production" inventory outputs and produce contradictory review markers.

## Scope
In scope:
- Harden `#[cfg(test)]` item skipping in:
  - `scripts/ci/check_no_production_expect.py`
  - `crates/kamn-core/tests/review_r53_docs_contract.rs`
- Add regression fixtures proving brace-heavy test modules do not leak test-only `expect()`.
- Align R55 marker formula text with implemented inventory semantics.

Out of scope:
- Bulk migration/removal of all `expect()` callsites repository-wide.
- Rewriting historical commit classifications.
- New external dependencies.

## Acceptance Criteria
- AC-1: Checker scanner skips `#[cfg(test)]` items even when those items contain brace-heavy string
  literals (raw strings / format braces).
- AC-2: R55 production `expect()` inventory in docs-contract tests uses the same robust skipping
  semantics as checker logic.
- AC-3: Regression fixture fails on old logic and passes on fixed logic.
- AC-4: R55 marker formula text matches implemented semantics (including cfg(test)-item skipping).
- AC-5: Targeted verification lanes pass (`scripts/ci/test_check_no_production_expect.sh` and
  `cargo test -p kamn-core --test review_r53_docs_contract`).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | Rust fixture with `#[cfg(test)] mod tests { ... "{...}" ... expect(...) }` | scanner skips entire test item; no production expect violation |
| C-02 | AC-1 | Regression | Existing checker harness + new fixture | harness remains green; fixture proves no leakage |
| C-03 | AC-2 | Conformance | R55 docs-contract inventory computation | measured inventory equals marker snapshot |
| C-04 | AC-4 | Conformance | R55 formula marker | text equals implemented semantics string |
| C-05 | AC-5 | Integration | checker harness + review docs-contract lane | both commands pass |

## Test Mapping
- `bash scripts/ci/test_check_no_production_expect.sh`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics / Observable Signals
- No false positives from brace-heavy cfg(test) blocks in production panic/expect scanning.
- Review marker formulas and computed values are semantically aligned.
- Deterministic inventory count remains stable across checker/docs-contract surfaces.
