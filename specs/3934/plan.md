# Issue #3934 Plan

- Issue: #3934
- Status: Implemented
- Spec: `specs/3934/spec.md`

## Delivery Approach
1. Execute panic-path runtime retirement in `#3936`:
   - retire `unreachable!()` path and reinforce signer typed-error flow (`#5153`)
   - harden production-source extraction for panic guards and broaden runtime coverage (`#5154`)
2. Execute panic-policy governance in `#3937`:
   - close checker false-negative gap (`#5156`)
   - enforce docs marker/remediation parity via docs-contract tests (`#5157`)
3. Close parent lineage artifacts for tasks and story.

## Affected Modules
- Runtime regression surfaces:
  - `crates/kamn-node/src/signer.rs`
  - `crates/kamn-node/src/cli_tests.rs`
- CI/docs governance surfaces:
  - `scripts/ci/check_no_production_expect.py`
  - `scripts/ci/test_check_no_production_expect.sh`
  - `docs/foundation/runtime-watchdog-attestation.md`
  - `docs/ci/strategy.md`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: cfg(test)-prefix false negatives in panic checker and source extraction.
  - Mitigation: dedicated RED/GREEN regression fixtures in runtime and checker paths.
- Risk: docs contract drift from checker/remediation policy.
  - Mitigation: fail-closed docs-contract assertions.

## Contracts and Interfaces
- Panic checker taxonomy:
  - `kamn.ci.production-panic-replacement-reason-taxonomy.v1`
- Docs remediation markers:
  - `panic_path_policy_remediation_steps_version=v1`
  - `panic_path_policy_remediation_step_1..3`

## Verification Strategy
- Child PR verification evidence:
  - `#5153`, `#5154`, `#5156`, `#5157`
- Parent closeout PRs:
  - `#5155` (`#3936`), `#5158` (`#3937`)
- Story closeout validates AC and conformance traceability.
