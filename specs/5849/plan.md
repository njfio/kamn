# Plan: Issue #5849

## Approach
1. Add a new checker script (`scripts/ci/check_e2e_live_workflow_contract.py`) that parses `.github/workflows/e2e-live.yml` and enforces required markers.
2. Add a companion test harness (`scripts/ci/test_check_e2e_live_workflow_contract.sh`) with:
   - pass fixture (current workflow copy)
   - fail fixture(s) for missing live toggle and truncated scenario list.
3. Update `.github/workflows/e2e-live.yml` SDK-direct scenario list to S-01..S-15.
4. Wire checker tests into `scripts/ci/test_ci_tools.sh` and add docs markers in `docs/ci/strategy.md`.
5. Run targeted regression commands and CI fast-mode suite.

## Affected Modules
- `.github/workflows/e2e-live.yml`
- `scripts/ci/check_e2e_live_workflow_contract.py` (new)
- `scripts/ci/test_check_e2e_live_workflow_contract.sh` (new)
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`

## Risks & Mitigations
- Risk: YAML parsing brittleness if checker uses naive grep.
  - Mitigation: use deterministic string marker checks with explicit reason codes and focused invariants.
- Risk: Expanding SDK-direct scenario set increases scheduled workflow runtime.
  - Mitigation: keep change to scheduled/dispatch workflow only; no fast-gate impact.

## Interfaces / Contracts
- Checker stdout contract:
  - `status=pass|fail`
  - `final_decision=GO|NO-GO`
  - `reason_taxonomy_version=<version>`
  - `reason_codes_csv=<csv>`
  - `reason_codes_value=none|<csv>`
- Deterministic reason codes for missing live markers and truncated scenario set.

## ADR
- Not required (no new dependency or protocol change).
