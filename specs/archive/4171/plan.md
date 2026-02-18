# Plan — Issue #4171

## Approach

1. Implement `scripts/ci/check_custody_rotation_ci_smoke_convergence.py` using the established
   convergence-checker pattern:
   - read workflow, ci-tools, strategy, and plan inputs
   - validate required smoke composition commands
   - validate local-heavy exclusion boundaries
   - validate docs marker lineage parity
   - emit deterministic reason taxonomy/report fields
2. Add `scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh`:
   - baseline pass fixture
   - missing smoke-composition fixtures
   - local-heavy leakage fixtures
   - strategy/plan drift fixtures
   - max-seconds overflow fixture
3. Wire the new test script into `scripts/ci/test_ci_tools.sh` in fast and full modes.

## Affected Modules

- `scripts/ci/check_custody_rotation_ci_smoke_convergence.py`
- `scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`

## Risks / Mitigations

- Risk: brittle checker due to prose-sensitive matching.
  Mitigation: enforce stable marker keys and command strings, not paragraph wording.
- Risk: false positives from scanning full ci-tools file.
  Mitigation: scope smoke-composition checks to extracted fast-mode block.
- Risk: local-heavy leakage rule gaps.
  Mitigation: explicitly check both workflow and ci-tools fast block for known local-heavy command.

## Interfaces / Contracts

- Checker output markers:
  - `status`, `final_decision`
  - `reason_taxonomy_version`
  - `reason_codes_csv`, `reason_codes_value`
  - `custody_rotation_ci_smoke_convergence_status`
  - `custody_rotation_ci_smoke_max_seconds`
  - `custody_rotation_local_heavy_max_seconds`
  - `custody_rotation_ci_smoke_lane_cost_profile`
  - `custody_rotation_local_heavy_execution_mode`
- JSON report schema:
  - `kamn.ci.custody-rotation-ci-smoke-convergence-report.v1`

## ADR

No ADR required. This is CI governance checker/test/docs wiring with no protocol/dependency change.
