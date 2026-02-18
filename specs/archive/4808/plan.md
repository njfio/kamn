# Plan — Issue #4808

## Approach

- Execute through two implementation tasks:
  - `#4813`: parameterize wave/matrix scripts and remove duplicate family implementations.
  - `#4814`: deploy shared test harness + JSON helper utilities and migrate high-duplication cohorts.
- Keep contract lane outputs stable while replacing boilerplate with shared helper/runners.
- Validate each subtask with RED->GREEN evidence and full `scripts/ci/test_ci_tools.sh` regression.

## Affected Modules

- Parameterized runner scripts under `scripts/ci/` and `scripts/framework/` (task `#4813`).
- Shared test harness and migrated wrapper-family tests (subtask `#4825`).
- Shared JSON helper primitives/command and migrated manual JSON writers (subtask `#4826`).

## Risks / Mitigations

- Risk: migration drift across numerous shell scripts.
  Mitigation: phased subtask delivery with dedicated migration contract tests and full CI regression gates.
- Risk: policy/output contract regression.
  Mitigation: preserve key=value/reason taxonomy markers and verify via CI tool suite.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Maintain stable key=value outputs and reason taxonomy markers.

## ADR

- Not required (no dependency/protocol/architecture changes in story closeout).
