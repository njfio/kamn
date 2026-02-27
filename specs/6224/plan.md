# Issue 6224 Plan

## Approach
1. Add module-local tests in `kamn-runtime-guards` for quota and fairness policy decision paths and marker helpers.
2. Add compatibility tests in `kamn-core` quota/fairness re-export modules to lock public API behavior through re-export boundaries.
3. Run targeted tests for fast iteration, then run crate-level tests for both affected crates.

## Affected Modules
- `crates/kamn-runtime-guards/src/quota_policy.rs`
- `crates/kamn-runtime-guards/src/fairness_policy.rs`
- `crates/kamn-core/src/quota_policy.rs`
- `crates/kamn-core/src/fairness_policy.rs`

## Risks and Mitigations
- Risk: assertions mismatch deterministic reason markers.
  - Mitigation: assert both enum variant and `as_str()` marker values.
- Risk: compatibility re-export drift in `kamn-core`.
  - Mitigation: add direct `kamn-core` tests invoking re-exported APIs.

## Interfaces
- Public quota/fairness policy functions and reason-marker constants in `kamn-runtime-guards`.
- Re-export compatibility layer in `kamn-core`.
