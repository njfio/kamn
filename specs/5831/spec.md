# Spec: Issue #5831 - Close R55 Unresolved Gaps With Enforced Contracts and Runtime Surface Reactivation

- Issue: #5831
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r55-review-gap-closure-and-runtime-surface-reactivation/index.md`
- Last Updated: 2026-02-23

## Problem Statement
`docs/review/gaps-and-issues-r55.md` lists five unresolved/high-risk gaps:
1. Governance remains structurally coupled to feature delivery (~55% governance commits).
2. Declared doc-contract cap lock (110) is breached (140).
3. `kamn-node`/`kamn-kolme` are reported frozen for six reviews.
4. Production `expect()` inventory (418) was reported without deterministic contract enforcement.
5. Spec-dir contamination recurrence indicates tracked-only counting is not uniformly applied.

These need implementation-level closure: code changes, integration, deterministic counting, and review-doc contract verification.

## Scope
In scope:
- Fix remaining review docs-contract counting paths to tracked-only `specs/` semantics.
- Enforce workspace-wide contract-file counting formula that matches R55 cap metrics.
- Add explicit R55 unresolved-item closure markers and consistency checks.
- Implement real integrated runtime-surface changes across `kamn-kolme` and `kamn-node` for service-auth scope taxonomy.
- Add deterministic production `expect()` inventory markers and validation checks in review docs-contract lanes.

Out of scope:
- Bulk rewrite eliminating all production `expect()` call sites.
- Historic commit history rewriting/reclassification.
- New external dependencies.

## Acceptance Criteria
- AC-1: Review docs-contract tests use tracked-only top-level spec-directory counting; untracked `specs/*` directories do not trigger false failures.
- AC-2: Review docs-contract tests compute workspace contract-file count with deterministic formula and validate cap status/mitigation markers in R55.
- AC-3: `kamn-kolme` defines shared service-auth scope taxonomy and `kamn-node` uses it for route-scope parsing/enforcement with regression tests.
- AC-4: R55 includes deterministic production `expect()` audit markers with formula, snapshot count, and policy status; docs-contract tests validate marker consistency.
- AC-5: R55 includes unresolved-item closure markers for all listed issues; docs-contract tests verify internal arithmetic/status invariants.
- AC-6: fmt, clippy (`-D warnings`), targeted docs-contract lanes, and full workspace tests are green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | untracked `specs/untracked-*` dir during docs-contract run | tracked spec count unchanged; lane passes |
| C-02 | AC-2 | Conformance | workspace contract-file formula over `crates/*/tests` | count and cap markers are consistent and policy-valid |
| C-03 | AC-3 | Unit/Functional | scope parsing/mapping in `kamn-kolme` + `kamn-node` | canonical scope values accepted; invalid/mismatch fails closed |
| C-04 | AC-4 | Conformance | R55 production expect marker set | measured count/ratio fields are parseable and self-consistent |
| C-05 | AC-5 | Conformance | R55 unresolved closure marker set | total/resolved/status and per-gap statuses are consistent |
| C-06 | AC-6 | Regression | workspace quality gates | fmt/clippy/test lanes pass |

## Test Mapping
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `cargo test -p kamn-node service_api_endpoint_tests -- --nocapture`
- `cargo test -p kamn-kolme -- --nocapture`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test`

## Success Metrics / Observable Signals
- No recurring false-positive spec-dir contamination failures from untracked `specs/` folders.
- R55 unresolved closure markers report full closure and pass docs-contract invariants.
- `kamn-node` and `kamn-kolme` both receive integrated production-surface updates in this wave.
- Production `expect()` audit is deterministic and contract-validated rather than ad-hoc.
