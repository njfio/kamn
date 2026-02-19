# Issue #5188 Plan

- Issue: #5188
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Approach
1. Build a Rust-native public API scanner in a new `kamn-core` integration test module:
   - Enumerate public modules from `crates/kamn-core/src/lib.rs`.
   - Count non-`pub(crate)` API declaration lines per module from owned source files.
   - Produce deterministic report output (stable schema + sorted module list + delta fields).
2. Add fixtures:
   - Baseline fixture with total + per-module counts.
   - Threshold fixture for warn/fail deltas and optional waiver path contract.
3. Implement fail-closed policy check in tests:
   - `within` and `warn` pass with explicit markers.
   - `fail` panics unless waiver is valid and scoped.
4. Update documentation and docs-contract coverage:
   - CI strategy command for report emission.
   - Baseline refresh steps.
   - Waiver requirements including mitigation issue marker.

## Affected Modules / Files
- `crates/kamn-core/tests/public_api_surface_policy.rs` (new)
- `fixtures/ci/kamn_core_public_api_surface_baseline.env` (new)
- `.ci/kamn-core-public-api-surface-thresholds.env` (new)
- `docs/ci/strategy.md` (update)
- `crates/kamn-core/tests/ci_strategy_docs.rs` (update)
- `specs/5188/*` (new)

## Risks and Mitigations
- Risk: scanner over/under-count drift from formatting differences.
  - Mitigation: strict token match rules + deterministic sorting + explicit schema version marker.
- Risk: policy thresholds too tight and block benign merges.
  - Mitigation: warn/fail split plus explicit waiver file contract with mitigation issue linkage.
- Risk: shell LOC regression via CI hook implementation.
  - Mitigation: Rust-test-only gate; no new shell wrappers.

## Interfaces / Contracts
- Baseline schema marker: `kamn.core.public-api-surface-baseline.v1`
- Report schema marker: `kamn.core.public-api-surface-report.v1`
- Policy marker: `kamn.core.public-api-surface-ratchet.v1`
- Deterministic keys:
  - `total_public_items`, `baseline_total_public_items`, `public_items_delta`
  - `module_public_items.<module>`
  - `module_public_items_delta.<module>`
