# Spec: Issue #5785 — Finalize R53 Milestone Closure Markers and Docs-Contract Guard

- Issue: #5785
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P2
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
`specs/milestones/r53-e2e-scenario-execution-activation/index.md` is stale: it still declares `Active issue(s): #5626` and marks delivery slice 4 as in progress, even though all R53 delivery issues (`#5620`, `#5622`, `#5624`, `#5626`) are closed with `status:done`. This leaves milestone closure state inconsistent and unguarded against regression.

## Scope
- Update R53 milestone index closure markers to reflect completed state.
- Add fail-closed docs-contract assertion(s) that require R53 milestone closure markers.
- Keep existing R53 docs/research references and issue IDs intact.

## Out of Scope
- Changes to runtime behavior or e2e scenario execution semantics.
- Rewriting `docs/review/gaps-and-issues-r53.md` snapshot tables/markers.
- Shell/workflow/template surface changes.

## Acceptance Criteria

### AC-1: R53 milestone index reflects closure-complete state
Given all R53 delivery issues are closed,
When reading `specs/milestones/r53-e2e-scenario-execution-activation/index.md`,
Then it must contain `Active issue(s): None`, include `#5626` in completed issues, and mark delivery slice 4 as completed.

### AC-2: Docs-contract coverage fails closed for stale R53 closure markers
Given docs-contract tests run,
When R53 milestone index regresses to an active issue/in-progress slice,
Then tests fail with deterministic assertions.

### AC-3: Existing R53 milestone issue references remain preserved
Given the milestone index is updated,
When docs-contract tests validate issue references,
Then all R53 slice issue IDs (`#5620`, `#5622`, `#5624`, `#5626`) remain referenced.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | `docs_contract_release_group` asserts `Active issue(s): None`, completed issue list includes `#5626`, and slice 4 shows `(Completed)`. |
| C-02 | AC-2 | Regression | RED run fails before milestone index update; GREEN run passes after update. |
| C-03 | AC-3 | Integration | Full `docs_contract_release_group` test target passes with all R53 issue ID references retained. |

## Success Metrics / Observable Signals
- Targeted RED then GREEN evidence captured for R53 closure assertion.
- `cargo test -p kamn-e2e-harness --test docs_contract_release_group` passes.
- `cargo fmt --all --check` and targeted clippy lane for e2e harness tests pass.
