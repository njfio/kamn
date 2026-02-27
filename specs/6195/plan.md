# Plan: Issue 6195 - Add Baseline Unit Coverage for Data Layer M0-M5

- Issue: #6195
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Add focused unit-test modules to each M0-M5 file, using local fixtures only.
2. Keep tests deterministic and lightweight:
   - no external IO,
   - no network,
   - no runtime process spawning.
3. Validate both success and fail-closed branches for key API entry points.
4. Run scoped `kamn-core` tests with filters for new module tests.

## Affected Modules

- `crates/kamn-core/src/data_layer_m0.rs`
- `crates/kamn-core/src/data_layer_m1.rs`
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/src/data_layer_m4_escrow_integration.rs`
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs`

## Risks and Mitigations

1. Risk: fixture setup complexity for canonical envelopes and escrow/vector inputs.
   - Mitigation: add compact local fixture helpers per module.
2. Risk: brittle assertions tied to unstable strings.
   - Mitigation: assert typed outcomes/variants and key invariant markers instead of broad string matching.
