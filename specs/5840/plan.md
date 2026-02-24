# Plan: Issue #5840

## Approach
1. Add parser-state helpers in Python checker to count braces only in code context, not inside strings/comments/raw strings.
2. Port equivalent parser-state logic into `review_r53_docs_contract.rs` cfg(test)-item skipper for parity.
3. Add regression fixtures in shell checker tests and Rust docs-contract tests for brace-heavy cfg(test) modules.
4. Update review marker formula text/expectations to explicitly describe cfg(test)-item stripping.

## Affected Modules
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `docs/review/gaps-and-issues-r55.md`

## Risks and Mitigations
- Risk: parser complexity introduces false negatives in panic-path checks.
  - Mitigation: add targeted fixtures covering strings/raw strings/comments plus existing failure fixtures.
- Risk: docs-contract marker expectation drift.
  - Mitigation: update marker line and exact-string assertion in same change.

## Interface/Contract Changes
- No public API change.
- Internal contract semantics clarified: production expect inventory is computed after cfg(test)-item stripping with literal/comment-aware brace scanning.

## ADR
- Not required (no architectural dependency or protocol change).
