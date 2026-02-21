# Issue #4004 Plan

## Approach

1. Implement `scripts/runtime/local_heavy_capacity_load_lane_contract.py` with:
   - deterministic baseline/fault profile fixtures,
   - throughput/latency/error markers,
   - versioned schema/taxonomy markers,
   - bounded runtime and fail-closed profile validation.
2. Add exec-dispatch wrapper `scripts/runtime/run_local_heavy_capacity_load_lane.sh` and registry mapping entry.
3. Add `crates/kamn-core/tests/local_heavy_capacity_load_lane_contract.rs` for Unit/Functional/Integration/Regression/Performance coverage.
4. Update `docs/ops/configuration.md` with marker expectations and command references.
5. Add docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.

## Affected Modules

- `scripts/runtime/local_heavy_capacity_load_lane_contract.py` (new)
- `scripts/runtime/run_local_heavy_capacity_load_lane.sh` (new symlink wrapper)
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/local_heavy_capacity_load_lane_contract.rs` (new)
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4004/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations

- Risk: local-heavy runner markers drift from docs/test contracts.
  - Mitigation: add strict docs-contract assertions for all required markers.
- Risk: adding wrapper without registry mapping causes runtime failures.
  - Mitigation: wire wrapper path in `scripts/lib/exec_registry.json` and verify via test invocation.
- Risk: scope creep into unrelated local-heavy lanes.
  - Mitigation: isolate to new runner path and targeted tests/docs only.

## Interfaces and Contracts

- New lane report schema:
  - `kamn.runtime.local-heavy-capacity-load-lane-report.v1`
- New artifact schema marker:
  - `kamn.runtime.local-heavy-capacity-load-artifact-schema.v1`
- New reason taxonomy marker:
  - `kamn.runtime.local-heavy-capacity-load-reason-taxonomy.v1`
