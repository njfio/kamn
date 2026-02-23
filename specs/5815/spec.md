# Spec: Issue #5815 - Close Residual S-03 sdk-direct Mutation Escapes

- Issue: #5815
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Issue #5814 reduced mutation escapes to 2/30, but two escaped mutants remain in `sdk_direct` S-03 mismatch guards (`query_message.message_id` and `list_messages.channel_id`) because the current live probe path has no deterministic injectable seam for response-shape assertions.

## Scope
In scope:
- Add a minimal injectable/test seam for S-03 sdk-direct response checks.
- Add deterministic unit/conformance tests that fail when S-03 query/list mismatch guards are inverted.
- Keep runtime behavior unchanged for production live probes.

Out of scope:
- Protocol/wire-format changes.
- Non-S-03 scenario behavior changes.
- Shell/workflow/template changes.

## Acceptance Criteria
- AC-1: sdk-direct S-03 helper fails closed when queried message_id differs from the sent message_id via deterministic unit seam.
- AC-2: sdk-direct S-03 helper fails closed when listed channel_id differs from created channel_id via deterministic unit seam.
- AC-3: Existing sdk-direct/driver live behavior and regressions remain green.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Injected S-03 probe path with mismatched query message_id | Returns `Err(..mismatched message_id..)`. |
| C-02 | AC-2 | Functional | Injected S-03 probe path with mismatched list channel_id | Returns `Err(..mismatched channel_id..)`. |
| C-03 | AC-3 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | Existing suites remain green. |

## Test Mapping
- `cargo test -p kamn-e2e-harness drivers::sdk_direct::tests::unit_run_live_s03_group_channel_probe_rejects_query_message_id_mismatch -- --nocapture`
- `cargo test -p kamn-e2e-harness drivers::sdk_direct::tests::unit_run_live_s03_group_channel_probe_rejects_list_channel_id_mismatch -- --nocapture`
- `cargo test -p kamn-e2e-harness -- --nocapture`

## Success Metrics / Observable Signals
- New deterministic sdk-direct S-03 mismatch tests pass.
- `cargo mutants --in-diff` no longer reports the two escaped sdk-direct S-03 mismatch mutants.
