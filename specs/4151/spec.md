# Issue #4151 Spec

- Title: Subtask: implement preflight fail-closed checker outputs and runbook marker synchronization
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md

## Problem Statement
Deployment preflight checker output and runbook marker contracts can drift, creating ambiguous remediation outcomes and weakening deterministic GO/NO-GO enforcement.

## Acceptance Criteria
- AC-1: Deployment preflight checker emits deterministic fail-closed marker contract outputs and reason-code taxonomy fields for marker/schema mismatches.
- AC-2: Runbook parity markers for deployment preflight checker output remain synchronized in `docs/deploy/kolme_devnet_ops.md` and `docs/planning/kolme-devnet-ops.md`.
- AC-3: Integration validation confirms stable output behavior across GO and NO-GO fixture paths.

## Scope
In scope:
- `scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py` output + fail-closed reason classification updates.
- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` validation updates for new marker/runbook outputs.
- Docs/runbook marker synchronization updates in deploy/planning docs plus Rust docs-contract tests.
- Lifecycle artifacts for issue `#4151`.

Out of scope:
- Deployment lane runtime orchestration redesign.
- New dependencies or protocol/wire-format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run preflight policy checker GO fixture | Deterministic marker-contract + schema/runbook parity outputs are emitted with `verified` status |
| C-02 | AC-1 | Regression | Run preflight policy checker NO-GO mismatch fixtures | Deterministic fail-closed marker/schema mismatch reason codes are emitted |
| C-03 | AC-2 | Conformance | Validate deploy/planning docs parity markers | Runbook marker contract strings remain synchronized with checker output |
| C-04 | AC-3 | Integration | Run shell contract lane + Rust docs tests | Stable pass/fail behavior remains deterministic end-to-end |

## Test Mapping
- `bash scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `cargo test -p kamn-core --test kolme_devnet_ops_docs`
- `cargo test -p kamn-core --test release_gonogo_checklist_docs`

## AC Verification
| AC | ✅/❌ | Test(s) |
|---|---|---|
| AC-1 | ✅ | preflight checker shell contract lane + GO/NO-GO fixture assertions |
| AC-2 | ✅ | deploy/planning docs markers + `kolme_devnet_ops_docs` assertions |
| AC-3 | ✅ | shell contract lane + Rust docs suites pass deterministically |

## Success Metrics
- Issue `#4151` closes with deterministic preflight marker/schema/runbook outputs enforced by tests.
- Deploy and planning runbooks remain synchronized with checker output marker contract.
