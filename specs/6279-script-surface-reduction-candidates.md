# Issue 6279 - Add enforced short-wrapper script reduction candidate matrix

## Objective
Create a canonical, fail-closed short-wrapper candidate matrix so script-surface
reduction can target high-volume consolidation opportunities with deterministic
selection rules.

## Inputs/Outputs
- Inputs:
  - Script inventory under `scripts/` (`*.sh` and `*.py`).
  - Existing script inventory baseline: `docs/developer/script-surface-index.md`.
  - Existing developer command surface: `docs/developer/readme-contract-reference.md`.
- Outputs:
  - New `docs/developer/script-surface-reduction-candidates.md` with deterministic
    markers and category-ranked candidate matrix.
  - New docs-contract test file enforcing marker presence and count consistency.
  - Discoverability link from `docs/developer/readme-contract-reference.md`.

## Boundaries/Non-goals
- In scope:
  - Candidate ranking documentation and fail-closed contract enforcement.
  - Reproducible command recipe for regenerating candidate counts.
- Out of scope:
  - Deleting scripts.
  - Script behavior refactors.
  - CI workflow topology changes.
  - New dependencies.

## Failure modes
- FM1: Candidate matrix exists but lacks deterministic threshold markers.
- FM2: Candidate counts drift from filesystem inventory without test detection.
- FM3: Candidate matrix is not discoverable from core developer contract docs.
- FM4: Regeneration commands are absent or non-reproducible.

## Acceptance criteria (testable booleans)
- AC1: `docs/developer/script-surface-reduction-candidates.md` includes schema markers,
  threshold markers, and a candidate matrix table.
- AC2: `crates/kamn-core/tests/script_surface_reduction_candidates_docs.rs` fails
  closed on missing markers or count mismatch vs filesystem inventory.
- AC3: Candidate matrix includes reproducible commands and current threshold counts:
  - shell short-wrapper threshold: `<= 25` LOC
  - python short-wrapper threshold: `<= 40` LOC
- AC4: `docs/developer/readme-contract-reference.md` links to the candidate matrix doc.

## Files to touch
- `docs/developer/script-surface-reduction-candidates.md` (new)
- `crates/kamn-core/tests/script_surface_reduction_candidates_docs.rs` (new)
- `docs/developer/readme-contract-reference.md`

## Error semantics
- Missing/invalid numeric markers are hard test failures.
- Filesystem mismatch against candidate matrix counts is a hard test failure.
- Missing discoverability link is a hard test failure.

## Test plan
- RED:
  - Add docs-contract tests requiring candidate markers, threshold markers, and
    filesystem-aligned counts.
  - Run targeted test and confirm failure before doc/link creation.
- GREEN:
  - Add candidate matrix doc and readme link to satisfy tests.
  - Re-run targeted docs-contract tests.
- REFACTOR:
  - Keep parser helpers small and deterministic.
- INTEGRATION:
  - Run adjacent readme/docs contract suites and full `make test`.

## Deviations
- None.
