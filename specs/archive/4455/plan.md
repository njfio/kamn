# Plan: Issue #4455

Status: Completed
Issue: #4455

## Approach

1. Add RED assertions in `scripts/ci/test_check_no_production_expect.sh` for new taxonomy and
   runtime evidence markers.
2. Implement deterministic reason mapping + runtime evidence normalization in
   `scripts/ci/check_no_production_expect.py`.
3. Extend secure-coding docs markers and docs contract assertions.
4. Add release-go/no-go checklist panic taxonomy section and docs contract assertions.
5. Run RED/GREEN loops and scoped hygiene checks.

## Affected Modules

- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `docs/security/secure-coding.md`
- `crates/kamn-core/tests/secure_coding_docs.rs`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `specs/4455/*`

## Risks and Mitigations

- Risk: changing checker output could break existing consumers.
  - Mitigation: preserve existing `status`/`violation_count`/`violation=` markers while adding
    new deterministic taxonomy/evidence markers.
- Risk: docs checklist drift due large file size.
  - Mitigation: add a focused test function for panic taxonomy markers.

## Interfaces / Contracts

- Checker adds deterministic marker fields:
  - `reason_taxonomy_version`
  - `reason_codes_csv`
  - `reason_codes_value`
  - `reason_class`
  - `runtime_panic_replacement_evidence_status`
  - `runtime_panic_replacement_evidence_violation_count`
  - `runtime_panic_replacement_evidence_files_csv`
  - `runtime_panic_replacement_evidence_outputs_csv`

## ADR

Not required: no dependency or architecture changes.
