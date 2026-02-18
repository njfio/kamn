# Issue #4955 Spec

- Title: Story: execute explicit legacy-script deletion waves with supersession contracts
- Status: Implemented
- Type: story
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Legacy wrapper/entrypoint scripts needed deterministic supersession evidence, deletion-wave activation, and stale-reference protection to reduce shell maintenance surface safely.

## Acceptance Criteria
- AC-1: Deletion-manifest contracts identify superseded scripts deterministically from inventory evidence.
- AC-2: At least one explicit deletion wave is activated across scoped domains without contract regression.
- AC-3: CI/local checks fail when deleted/superseded entrypoints remain referenced.
- AC-4: Story-level lifecycle artifacts map all child task deliveries and conformance evidence.

## Scope
In scope:
- Superseded inventory + deletion manifest contracts.
- First deletion wave activation for runtime/kolme and canary/ci/deploy/governance families.
- Stale-reference detector and CI fail-closed wiring.

Out of scope:
- Additional deletion waves beyond first-wave task scope.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh` | deterministic inventory/manifest parity |
| C-02 | AC-2 | Integration | first-wave task deliveries (`#4958`, `#4959`) | wave activation evidence merged |
| C-03 | AC-3 | Regression | `bash scripts/ci/test_check_stale_script_references.sh` | stale references fail closed |
| C-04 | AC-4 | Functional | updated story lifecycle docs | child-task evidence mapped and synchronized |

## Test Mapping
- AC-1: `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- AC-2: `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
- AC-3: `bash scripts/ci/test_check_stale_script_references.sh`
- AC-4: child task closure mapping (`#4958`, `#4959`, `#4960`)

## Success Metrics
- First deletion-wave story scope completed with deterministic contracts and stale-reference guardrails.
