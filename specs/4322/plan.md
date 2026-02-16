# Issue #4322 Plan

- Issue: `#4322`
- Status: `Completed`

## Approach
- Add a small durable commit checker projection surface in `block_pipeline` with:
  - deterministic taxonomy version marker,
  - reason projection classing for replay drift/commit store/lane-boundary errors,
  - lane-boundary enforcement for `CiSmoke` and `LocalHeavy` modes.
- Keep implementation in Rust only to avoid introducing new shell wrapper families.
- Add a dedicated conformance test target for required categories.
- Extend `docs/ci/strategy.md` and docs tests with durable commit checker boundary markers.

## Affected Modules
- `crates/kamn-core/src/block_pipeline.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/block_commit_checker_reason_mapping.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: reason-class mapping drifts from existing `BlockPipelineError::reason_code()` outputs.
- Mitigation: projection uses existing reason codes and fail-closed defaults plus regression tests.
- Risk: boundary checks become script-coupled.
- Mitigation: boundary contract remains pure Rust and documentation-driven.

## Interface Contract
- New public API in `block_pipeline` for durable commit reason projection and lane-boundary enforcement.
- No protocol/wire format changes.

## ADR
- Not required.
