# Spec: Issue #5875 - Immutable Review Docs + Shell LOC Reduction

- Issue: #5875
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Post-publication review docs are still mutable in practice because existing checks can be updated alongside content edits. In parallel, shell-surface LOC remains materially high and needs measurable downward movement rather than static policy markers.

## Scope
In scope:
- Enforce fail-closed immutability contract checks for published review docs (effective from a defined release floor).
- Extend review-doc policy metadata so immutability requirements are explicit and machine-validated.
- Reduce tracked shell LOC with behavior-preserving consolidation/refactoring.
- Add regression tests that fail closed for review mutability and shell LOC ratchet breaches.

Out of scope:
- Retroactively rewriting previously published review files before the immutability effective-release floor.
- Runtime protocol/API behavior changes unrelated to review immutability or shell-surface reduction.

## Acceptance Criteria
### AC-1 Review-doc immutability contract is enforceable
Given tracked `docs/review/gaps-and-issues-r*.md` files at/after configured effective release,
When docs-contract tests run,
Then each applicable review doc must satisfy the immutability rule and fail if edited post-creation.

### AC-2 Immutability policy markers are explicit and validated
Given review freeze and moratorium policy docs,
When policy contract tests parse marker keys,
Then schema/version/effective-release/enforcement markers are present and internally consistent.

### AC-3 Shell LOC decreases against recorded baseline
Given baseline shell metric at issue start (`shell_line_total=122232`),
When shell-surface checks run after implementation,
Then tracked shell LOC is lower than baseline by at least the target delta and reported as improved.

### AC-4 No shell-lane behavior regressions
Given refactored shell assets,
When targeted script tests and affected policy checks run,
Then contract lane outcomes remain passing.

## Conformance Cases
- C-01 (Functional, AC-1): `review_r53_docs_contract` rejects immutability-policy violations for applicable release docs.
- C-02 (Functional, AC-2): policy markers for immutability schema/effective release/enforcement mode are required and validated.
- C-03 (Regression, AC-1): mutation scenario (post-creation edit on in-scope review doc) is detected and fails the contract.
- C-04 (Integration, AC-3): shell LOC report (`check_shell_loc_hard_ceiling`) shows `shell_line_total < 122232`.
- C-05 (Integration, AC-3): shell/rust ratio report remains within guardrails while reflecting lowered shell LOC.
- C-06 (Regression, AC-4): targeted script/contract tests for refactored shell assets remain green.

## Success Metrics / Observable Signals
- `cargo test -p kamn-core --test review_r53_docs_contract` passes with immutability checks enabled.
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh ...` reports reduced `shell_line_total`.
- `shell_surface_ratio_target_status=improved` is declared with measured actual deltas in PR/closure artifacts.
