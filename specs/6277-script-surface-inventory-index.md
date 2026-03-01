# Issue 6277 - Add enforced script-surface inventory index and discoverability link

## Objective
Introduce a canonical, fail-closed script-surface inventory document so contributors can quickly see current script footprint by category and track reduction work against a deterministic baseline.

## Inputs/Outputs
- Inputs:
  - Script tree under `scripts/` (`*.sh` and `*.py`).
  - Existing contributor contract reference: `docs/developer/readme-contract-reference.md`.
  - Existing docs-contract test patterns in `crates/kamn-core/tests`.
- Outputs:
  - New `docs/developer/script-surface-index.md` with schema markers and categorized counts.
  - New docs-contract test enforcing required markers/sections and count consistency.
  - Discoverability link from `docs/developer/readme-contract-reference.md`.

## Boundaries/Non-goals
- In scope:
  - Documentation inventory and deterministic doc-contract guardrails.
  - Regeneration command recipe for the inventory.
- Out of scope:
  - Any script behavior changes.
  - CI workflow changes.
  - Script-count reduction/refactors.
  - New dependencies.

## Failure modes
- FM1: Script inventory doc is missing required schema markers.
- FM2: Category totals drift from declared global totals without test detection.
- FM3: Contributors cannot discover the inventory from core developer contract docs.
- FM4: Regeneration steps are undocumented and inventory cannot be reproduced.

## Acceptance criteria (testable booleans)
- AC1: `docs/developer/script-surface-index.md` exists and includes deterministic schema and total-count markers.
- AC2: `crates/kamn-core/tests/script_surface_index_docs.rs` fails closed when required markers/sections are missing or inconsistent.
- AC3: `docs/developer/readme-contract-reference.md` links to `docs/developer/script-surface-index.md`.
- AC4: The index contains reproducible shell commands for recomputing totals and per-category counts.

## Files to touch
- `docs/developer/script-surface-index.md` (new)
- `crates/kamn-core/tests/script_surface_index_docs.rs` (new)
- `docs/developer/readme-contract-reference.md`

## Error semantics
- Missing schema markers, malformed numeric markers, or inconsistent counts are hard test failures.
- Missing discoverability link is a hard test failure.

## Test plan
- RED:
  - Add failing docs-contract tests that require index markers, count consistency, and readme link.
  - Run targeted test to confirm failure before docs updates.
- GREEN:
  - Add script-surface inventory doc and readme link to satisfy tests.
  - Re-run targeted docs test.
- REFACTOR:
  - Extract small parsing helpers to keep tests deterministic and maintainable.
- INTEGRATION:
  - Run adjacent docs-contract lane touching readme-contract references and targeted docs suites.

## Deviations
- None.
