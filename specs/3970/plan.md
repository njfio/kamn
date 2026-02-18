# Plan - Issue #3970

## Approach

1. Strengthen the non-Kolme dispatcher wrapper matrix test to include wave-1 canary wrappers in addition to governance wrappers.
2. Assert each wrapper is:
   - executable,
   - symlink-backed to `../framework/run_non_kolme_contract_lane_dispatch.sh`,
   - manifest-resolvable through shared dispatcher.
3. Keep deterministic unknown-wrapper fail-closed checks in the same matrix test.
4. Verify wave-1 baseline determinism and fast-gate integration with focused commands.

## Affected Paths

- `scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh`
- `docs/ci/strategy.md`
- `specs/3970/spec.md`
- `specs/3970/plan.md`
- `specs/3970/tasks.md`

## Risks / Mitigations

- Risk: broader matrix assertions could introduce false negatives if wrapper targets vary by directory.
  Mitigation: assert explicit expected symlink target and use basename when resolving manifest.

- Risk: docs and contract behavior drift.
  Mitigation: update strategy text in the same change and verify with strategy contract tests.

## ADR

- Not required (no new dependency, protocol, or architecture decision).
