# Issue #4959 Spec

- Title: Task: execute first deletion wave for superseded shell scripts with parity validation
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
The first deletion wave needed deterministic, fail-closed superseded-script contracts across selected domains, with manifest/inventory parity and stale-reference protection after wave activation.

## Acceptance Criteria
- AC-1: First deletion wave is executed for scoped domains with deterministic superseded inventory + deletion manifest coverage.
- AC-2: Deterministic fail-closed behavior is preserved for drift/regression scenarios.
- AC-3: Required Unit/Functional/Integration/Regression suites pass for wave contracts.
- AC-4: Documentation/process markers are synchronized with merged task/subtask delivery.

## Scope
In scope:
- Parent task consolidation of merged subtasks:
  - `#4970` runtime/kolme wrapper compaction + wave activation.
  - `#4969` canary/ci/deploy/governance wave activation + parity proofs.
- Verification that inventory/deletion-manifest contracts and stale-reference checks remain deterministic and green.

Out of scope:
- Subsequent deletion waves beyond the first-wave scope.
- Unrelated shell-surface initiatives outside this milestone.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh` | Wave inventory/manifest parity passes with expanded deterministic entries |
| C-02 | AC-2 | Regression | legacy inventory vs expanded manifest check | deterministic NO-GO with `superseded_deletion_manifest_references_unknown_script` |
| C-03 | AC-3 | Integration/Regression | `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` | CI governance suites remain green |
| C-04 | AC-4 | Functional/Regression | issue/process/spec closure markers | parent task markers and lifecycle docs are synchronized |

## Test Mapping
- AC-1: `scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- AC-2: `python3 scripts/ci/superseded_script_inventory.py check ...` red/green parity evidence
- AC-3: `scripts/ci/test_check_stale_script_references.sh`, `scripts/ci/test_ci_tools_command_surface_contract.sh`, fast-mode `scripts/ci/test_ci_tools.sh`
- AC-4: closure evidence in issue `#4959` + updated lifecycle docs

## Success Metrics
- First-wave superseded inventory/deletion-manifest contracts are active and deterministic across runtime/kolme and canary/ci/deploy/governance domains.
- Required regression suites pass with no fail-closed policy regressions.
