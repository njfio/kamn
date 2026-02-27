# Spec: Issue #6232 - Compact README and Move Contract References to Docs

- Status: Implemented
- Priority: P2
- Parent: #6223
- Milestone: R59 Swarm Gap Closure

## Problem Statement

`README.md` has grown to onboarding-hostile size and mixes quickstart guidance with detailed contract markers. The repo needs a concise top-level entrypoint while preserving deterministic contract references in stable docs locations.

## Scope

In scope:
- Reduce `README.md` to <=200 lines.
- Keep quickstart, architecture map, and contributor entrypoints in README.
- Move contract-heavy command/reference material into a dedicated docs page and link from README.
- Preserve deterministic contract marker coverage by relocating checks to docs references where required.

Out of scope:
- Removing contract lanes.
- Changing runtime behavior.

## Acceptance Criteria

### AC-1 Compact README
Given the repository root README,
When counted line-by-line,
Then it contains 200 lines or fewer.

### AC-2 Onboarding Retained
Given a new contributor,
When reading README,
Then quickstart commands, architecture navigation, and contributor entrypoints are present.

### AC-3 Contract Reference Relocation
Given contract-heavy README markers,
When validating docs references,
Then those markers are maintained in docs pages referenced by README instead of expanded inline README blocks.

## Conformance Cases

- C-01 (AC-1, Unit): README line-count contract test fails above 200 and passes at/under 200.
- C-02 (AC-2, Functional): README contains required onboarding anchors and docs navigation links.
- C-03 (AC-3, Regression): Existing contract marker validations point to relocated docs references and remain deterministic.
