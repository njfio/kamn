# Issue #4969 Spec

- Title: Subtask: execute canary/ci/deploy/governance deletion wave with contract parity proof
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Wave-1 deletion contracts only covered `scripts/kolme/*` wrappers. Canary/CI/Deploy/Governance wrappers were still absent from superseded-inventory and deletion-manifest contracts, so the first multi-domain wave was incomplete.

## Acceptance Criteria
- AC-1: Superseded inventory includes canary/ci/deploy/governance contract-lane wrappers with deterministic replacement evidence and ownership attribution.
- AC-2: Deletion manifest wave includes those wrapper paths with valid reason codes and checker parity remains GO.
- AC-3: Regression proof exists that legacy inventory (kolme-only) fails against the expanded manifest while the updated baseline passes.
- AC-4: Contract tests enforce that canary/ci/deploy/governance wave entries remain present in the manifest.

## Scope
In scope:
- Expand migration-group fixture coverage to include canary/ci/deploy/governance wrapper lanes.
- Expand superseded lane ownership fixture for those families.
- Regenerate superseded inventory baseline and expand deletion manifest entries.
- Add regression assertion in manifest checker tests for non-kolme wave coverage.

Out of scope:
- Physical deletion of non-kolme wrapper files in this subtask.
- Runtime/kolme deletion wave work (handled by #4970).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh` | inventory generation + baseline parity succeeds with 40 superseded entries |
| C-02 | AC-2 | Functional/Regression | `python3 scripts/ci/superseded_script_inventory.py check ...` with updated baseline+manifest | `status=ok`, `final_decision=GO`, `reason_codes=none` |
| C-03 | AC-3 | Regression | checker run with temporary legacy (kolme-only) inventory vs expanded manifest | deterministic fail with `superseded_deletion_manifest_references_unknown_script` and `unknown_manifest_entry_count=15` |
| C-04 | AC-4 | Unit/Regression | `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh` | explicit assertion guarantees all 15 canary/ci/deploy/governance entries remain present |

## Test Mapping
- AC-1: `scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- AC-2: `scripts/ci/test_check_superseded_script_deletion_manifest.sh`, `python3 scripts/ci/superseded_script_inventory.py check ...`
- AC-3: temp legacy inventory red-run (`unknown_manifest_entry_count=15`) then green-run (`unknown_manifest_entry_count=0`)
- AC-4: new Python assertion block inside `scripts/ci/test_check_superseded_script_deletion_manifest.sh`

## Success Metrics
- Superseded inventory baseline expands from 25 to 40 deterministic entries.
- Deletion manifest expands from 25 to 40 deterministic entries.
- All scoped contract-lane regression suites remain green.
