# Plan — Issue #4814

## Approach

- Deliver through two child subtasks with strict RED->GREEN->Regression sequencing:
  - `#4825`: introduce `scripts/lib/test_harness.sh` and migrate first high-duplication wrapper-family test cohort.
  - `#4826`: introduce JSON helper primitives/command and migrate high-volume manual JSON write sites.
- Preserve existing lane contracts while replacing duplicated shell boilerplate with helper-based calls.
- Run full `scripts/ci/test_ci_tools.sh` after each subtask to ensure end-to-end compatibility.

## Affected Modules

- `scripts/lib/test_harness.sh`
- `scripts/lib/test_test_harness_migration_contract.sh`
- `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh`
- `scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/lib/common.sh` (JSON helpers)
- `scripts/lib/write_json_file.sh`
- `scripts/lib/test_json_write_helper_migration_contract.sh`
- 89 migrated shell scripts across `scripts/runtime`, `scripts/kolme`, `scripts/ci`, `scripts/sdk`, `scripts/deploy`, `scripts/channel`, `scripts/message`, `scripts/did`, `scripts/bridge`

## Risks / Mitigations

- Risk: migration drift in wrapper-family and policy/test scripts.
  Mitigation: dedicated migration contract tests for harness and JSON-helper adoption.
- Risk: unintended CI behavior drift from broad script edits.
  Mitigation: full `scripts/ci/test_ci_tools.sh` regression on each subtask PR before merge.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Keep key=value and reason-taxonomy output contracts unchanged.
- Add helper interface:
  - `bash scripts/lib/write_json_file.sh <output-json-path>` (JSON payload via stdin).

## ADR

- Not required (no dependency/protocol/architecture changes).
