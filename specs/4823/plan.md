# Plan — Issue #4823

## Approach

1. Add a new contract test that asserts desired end-state topology:
   - shared runner exists and is executable
   - wave10-wave19 entrypoints are symlinks to that runner
   - per-wave definition files exist and contain wrapper paths
2. Run the contract test before implementation to capture RED evidence.
3. Implement shared runner + wave definition files.
4. Replace wave10-wave19 script bodies with symlink entrypoints that preserve existing command names.
5. Re-run targeted wave matrix tests and full CI-tools regression for GREEN evidence.

## Affected Modules

- `scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave10_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave11_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave12_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave13_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave14_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave15_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave16_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave17_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave18_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/wave_definitions/non_kolme_wave10_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave11_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave12_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave13_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave14_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave15_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave16_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave17_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave18_lightweight_wrappers.txt`
- `scripts/framework/wave_definitions/non_kolme_wave19_lightweight_wrappers.txt`
- `scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh`
- `scripts/kolme/contracts/continuous_runtime_commit_contract_lane.py` (mergeability fix discovered during CI run)

## Risks / Mitigations

- Risk: CI command-surface drift if wave script names change.
  Mitigation: keep existing `wave10`-`wave19` filenames as entrypoint symlinks and validate with CI-tools contracts.
- Risk: wave definition data drift from expected wrappers.
  Mitigation: add explicit runner-topology contract test and keep one definition file per wave.
- Risk: integration reruns blocked by unrelated brittle test assertions.
  Mitigation: apply minimal deterministic pass-count matching fix when encountered (`continuous_runtime_commit_contract_lane.py`) and re-run full CI-tools regression.

## Interfaces / Contracts

- Preserve existing invocation interface:
  - `bash scripts/framework/test_non_kolme_wave${lightweight_wave}_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- Preserve deterministic dispatcher fallback marker checks:
  - `dispatch_status=fail`
  - `fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1`
  - `fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped`
  - `fallback_reason_code=dispatcher_unknown_wrapper`

## ADR

No ADR required; this is a bounded script deduplication refactor with no protocol/dependency changes.
