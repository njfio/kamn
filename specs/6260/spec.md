# Issue 6260 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6256

## Problem Statement
All workspace crates currently lack a `README.md`, and seven crates have no architecture documentation references under `docs/architecture`. This increases onboarding cost and weakens contract discoverability.

## Scope
In scope:
- Add `README.md` for every crate under `crates/`.
- Add architecture documents for the seven crates with zero existing architecture references.
- Update `docs/architecture/README.md` navigation index to include the new architecture docs.

Out of scope:
- Deep redesign of existing architecture docs.
- Functional code changes.

## Acceptance Criteria
- AC-1: Every crate directory under `crates/` contains `README.md`.
- AC-2: New architecture docs exist for crates with zero current references:
  - `kamn-agent-lib`
  - `kamn-cli`
  - `kamn-crypto`
  - `kamn-data-layer`
  - `kamn-e2e-harness`
  - `kamn-snapshot-journal`
  - `kamn-types`
- AC-3: `docs/architecture/README.md` links to all newly added architecture docs.
- AC-4: Docs quality gate/checkers used in CI remain passing for touched docs.

## Conformance Cases
- C-01 (AC-1, Conformance): `find crates -mindepth 1 -maxdepth 1 -type d | while read -r d; do test -f "$d/README.md" || echo "missing:$d"; done` emits no missing entries.
- C-02 (AC-2, Conformance): all seven expected architecture files exist under `docs/architecture/`.
- C-03 (AC-3, Functional): `docs/architecture/README.md` contains navigation links for each new architecture doc.
- C-04 (AC-4, Regression): docs-focused validation commands used by repo remain green.

## Test Mapping
- Functional/Conformance: file existence + index-link checks.
- Regression: run docs contract/check scripts when available.
- Unit/Property/Fuzz/Mutation/Performance: N/A (documentation-only task).
