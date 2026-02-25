# Plan: Issue #5989

## Approach
- Introduce shared helper script:
  - `scripts/ci/test_check_kamn_node_extraction_threshold_common.sh`
- Parameterize variant-specific settings:
  - checker path
  - threshold fixture path
  - threshold schema id
  - warn/fail reason codes
  - exception schema id
  - exception applied/expired reason codes
  - tracking issue
  - success message
- Replace duplicated script bodies with thin wrappers that invoke common harness.
- Run both wrapper test scripts to verify behavior parity.

## Affected Modules
- `scripts/ci/test_check_kamn_node_main_rs_extraction_threshold.sh`
- `scripts/ci/test_check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`
- `scripts/ci/test_check_kamn_node_extraction_threshold_common.sh` (new)

## Risks / Mitigations
- Risk: parameter mismatch changes assertion behavior.
  Mitigation: preserve exact previous constants via wrapper arguments.
- Risk: wrapper invocation drift.
  Mitigation: enforce strict argument parsing and fail-closed checks in common harness.

## Interfaces / Contracts
- Existing wrapper script paths remain unchanged.
- New shared internal shell harness consumed by those wrappers.
