# Plan: Issue #5831 - R55 Gap Closure Contracts and Runtime Surface Reactivation

- Issue: #5831
- Spec: `specs/5831/spec.md`
- Status: Implemented

## Approach
1. Patch review docs-contract counting helpers to use tracked-only spec discovery and workspace contract-file counting.
2. Add/extend R55 docs-contract assertions for unresolved closure markers, cap mitigation markers, and production `expect()` audit markers.
3. Implement shared service-auth scope taxonomy in `kamn-kolme` and integrate into `kamn-node` scope-policy enforcement.
4. Update `docs/review/gaps-and-issues-r55.md` with deterministic closure markers for all listed unresolved items.
5. Run RED->GREEN verification lanes and full quality gates.

## Affected Modules
- `docs/review/gaps-and-issues-r55.md`
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `crates/kamn-kolme/src/lib.rs`
- `crates/kamn-kolme/src/service_api_scope.rs` (new)
- `crates/kamn-node/Cargo.toml`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `specs/milestones/r55-review-gap-closure-and-runtime-surface-reactivation/index.md`
- `specs/5831/{spec,plan,tasks}.md`

## Interfaces / Contracts
- Tracked-only spec-directory formula: top-level directory set derived from `git ls-files specs`.
- Workspace contract-file formula: test files under `crates/*/tests` where filename contains `contract`.
- Service-auth scope taxonomy contract: normalized parse + canonical render from shared `kamn-kolme` type.

## Risks and Mitigations
- Risk: R55 marker arithmetic drifts from measured counts.
  - Mitigation: parseable numeric markers with explicit invariant assertions in docs-contract tests.
- Risk: Scope taxonomy integration breaks existing route auth behavior.
  - Mitigation: preserve scope strings and reason codes; add targeted route/parse regression tests.
- Risk: Existing broad workspace changes introduce unrelated failures.
  - Mitigation: run focused lanes first, then full workspace gate, and patch only scoped regressions.

## ADR
- Not required: no new external dependency or wire-format protocol change.
