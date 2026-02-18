# Issue #4154 Plan

- Issue: #4154
- Status: Implemented

## Approach
1. Extend existing milestone lineage test block in `scripts/deploy/test_generate_gonogo_evidence_bundle.sh` with targeted rollback-link and recovery-link incompleteness fixtures.
2. Add a tampered linked-artifact lineage contract marker fixture and assert both policy and lineage checker rejection.
3. Keep scope test-only to avoid changing runtime/policy behavior.

## Affected Modules
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `specs/4154/spec.md`
- `specs/4154/plan.md`
- `specs/4154/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Reuse deterministic fixture generation flow already used in the go/no-go bundle tests.
  - Assert exact reason-code markers to keep failure modes stable.
  - Run only targeted go/no-go evidence suite and formatting check for quick feedback.

## Interface Contract
- No new interfaces.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped subtask.
