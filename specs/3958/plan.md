# Issue #3958 Plan

- Issue: #3958
- Status: Implemented

## Approach
1. Extend the real-node profile policy checker output with explicit marker fields for quorum drift and signer disagreement status/go-no-go decisions.
2. Reuse existing signature decision reason codes to derive marker subsets deterministically (no new runtime reason generation path).
3. Extend existing shell contract tests to assert new marker fields on GO and NO-GO fixture paths.
4. Document marker contracts in `docs/ci/strategy.md` and enforce presence with a focused docs test in `crates/kamn-core/tests/ci_strategy_docs.rs`.

## Affected Modules
- `scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py`
- `scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: Marker fields drift from reason-code semantics.
  Mitigation: derive marker state directly from existing reason-code subsets, not parallel logic.
- Risk: Additional shell assertions inflate maintenance surface.
  Mitigation: extend existing test script blocks only; no new shell entrypoints.
- Risk: CI behavior changes unexpectedly due output key drift.
  Mitigation: preserve existing keys and add backward-compatible marker fields.

## Interfaces / Contracts
- Existing checker interface remains backward-compatible; new output keys are additive.
- No new dependencies and no workflow changes.
