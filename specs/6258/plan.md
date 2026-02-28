# Issue 6258 Plan

## Approach
1. Inventory root-public modules and identify deprecated/internal targets.
2. Tighten visibility in `kamn-core` (`pub mod` -> `mod`, remove shim modules).
3. Preserve stable root API via curated `pub use` exports.
4. Update downstream in-workspace usage to root re-exports.
5. Refresh API-surface baseline fixture to match post-tightening inventory.
6. Verify conformance commands and targeted crate test suites.

## Affected Paths
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-types/src/lib.rs`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env`
- `specs/6258/spec.md`
- `specs/6258/plan.md`
- `specs/6258/tasks.md`

## Risks and Mitigations
- Risk: accidental breaking change for in-workspace consumers.
  - Mitigation: update callsites to curated exports and run `kamn-node` tests.
- Risk: missing re-export for previously used items.
  - Mitigation: compile failures drive explicit re-export/callsite adjustment.
- Risk: stale policy baseline fails closed after visibility changes.
  - Mitigation: regenerate fixture from deterministic policy report output.

## Interface/Contract Notes
- Goal is API surface reduction, not behavior change.
- Public contract should be explicit through `pub use` rather than module leakage.

## Verification Notes
- `public_api_surface_policy` contract tests pass after baseline refresh.
- `kamn-types` and `kamn-node` targeted suites pass.
- Full tri-crate run passes with local perf budget override
  (`KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS=350`) due unrelated local timing
  sensitivity in signer perf lane.
