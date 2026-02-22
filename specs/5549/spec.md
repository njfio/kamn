# Issue #5549 Spec - Review Artifact Snapshot Semantics and Reconciliation-Loop Guardrails

- Status: Implemented
- Issue: #5549
- Parent: None
- Milestone: R50.40 Review artifact snapshot semantics and reconciliation-loop guardrails

## Problem Statement
R50 review findings identify a self-referential governance loop where branch-count marker reconciliation created repeated governance-only issues without capability gain.

## Scope
In scope:
- Define R50+ review snapshot semantics for volatile markers (branch/open-item counts) as point-in-time values.
- Encode branch-count marker contract mode as informational-only.
- Add docs-contract guardrails that cap branch reconciliation marker chains to one atomic follow-up issue.
- Update R50 review artifact markers to the new snapshot policy schema.

Out of scope:
- Runtime/service behavior changes.
- Protocol/wire/schema changes.
- Retrofitting every historical review artifact beyond policy-compatible contract updates.

## Acceptance Criteria
- AC-1: `docs/review/README.md` defines deterministic snapshot-semantics marker policy for R50+ review artifacts.
- AC-2: Branch-count markers are enforced as informational-only in docs-contract tests (no hard equality to live remote branch state).
- AC-3: Docs-contract tests enforce reconciliation-chain guardrails for R50+ review artifacts (`chain_count <= chain_max`, `chain_max = 1`).
- AC-4: `docs/review/gaps-and-issues-r50.md` includes deterministic snapshot-semantics markers using the new policy schema.
- AC-5: Review docs-contract suites remain green after migration.

## Conformance Cases
- C-01 (AC-1): review policy doc contains snapshot-semantics required keys and invariants.
- C-02 (AC-2): legacy R49 docs-contract test no longer enforces fixed branch-count values or reconciliation issue IDs.
- C-03 (AC-3): new R50+ docs-contract test fails when reconciliation chain count exceeds max or contract mode differs from informational-only.
- C-04 (AC-4): R50 review artifact exposes required snapshot-semantics markers with valid numeric values.
- C-05 (AC-5): targeted docs-contract lanes pass (R49 review contract, activity ratio contract, new snapshot contract).

## Success Metrics / Observable Signals
- No further branch-count reconciliation churn is needed for point-in-time review artifacts.
- R50+ review files carry explicit snapshot semantics with deterministic, testable guardrails.
- Existing review docs-contract coverage stays green.
