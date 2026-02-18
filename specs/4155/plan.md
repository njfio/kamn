# Issue #4155 Plan

## Objective
Lock deterministic mapping between upgrade-lineage failure reasons and promotion-gate reason-code outputs for rehearsal lineage failures.

## Approach
1. Extend existing rollback/recovery lineage-missing regression sections in `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
2. Assert explicit reason mapping fields from `check_upgrade_rehearsal_lineage_policy.py` output:
   - `upgrade_lineage_reason_codes_csv`
   - `upgrade_lineage_reason_codes_value`
   - `promotion_gate_reason_codes_csv`
   - `promotion_gate_reason_codes_value`
3. Update `docs/foundation/release-gonogo-checklist.md` with explicit mapping markers for rollback and recovery lineage-missing fail-closed decisions.
4. Run targeted deploy regression suite and governance stale-reference gate.

## Affected Modules
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `specs/4155/*`

## Risks and Mitigations
- Risk: brittle assertions if reason-code ordering drifts.
  - Mitigation: assert exact deterministic single-reason mapping for rollback/recovery-specific fixtures.
- Risk: doc/contract drift.
  - Mitigation: keep marker text aligned with checker output fields used in tests.

## Interfaces and Contracts
- `python3 scripts/deploy/check_upgrade_rehearsal_lineage_policy.py --bundle-file <bundle> --expected-final-decision NO-GO --require-reason-code <code>`
- Marker contract fields:
  - `upgrade_lineage_reason_codes_csv`
  - `upgrade_lineage_reason_codes_value`
  - `promotion_gate_reason_codes_csv`
  - `promotion_gate_reason_codes_value`

## ADR
- Not required (no dependency, protocol, or architecture change).
