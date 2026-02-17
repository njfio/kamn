# Plan — #4374

Status: Reviewed

## Approach

- Add RED tests for composite provider/signer evidence linkage gaps.
- Implement deterministic composite reason taxonomy and boundary marker outputs.
- Add fail-closed mismatch checks for partial/inconsistent evidence.
- Update docs and docs-contract tests for CI smoke/local-heavy composite gate boundaries.
- Validate with targeted suites and full repo gates.

## Affected Areas

- Runtime/CI contract scripts for live provider + signer gate checks.
- `docs/ci/strategy.md`
- Related docs-contract tests in `crates/kamn-core/tests/`.

## Risks and Mitigations

- Risk: marker drift between scripts/docs/tests.
  - Mitigation: co-land docs and docs-contract assertions with implementation.
- Risk: boundary enforcement could accidentally increase PR CI cost.
  - Mitigation: keep smoke path bounded and local-heavy explicit opt-in.
