# Issue #3944 Plan

- Issue: #3944
- Status: Completed
- Spec: `specs/3944/spec.md`

## Implementation Approach
1. Add a RED contract test in `main_module_extraction_contract.rs` that asserts runtime test shell boundaries (`#[test]` exclusion, include markers, line budget).
2. Split `runtime_tests.rs` test bodies into focused include fragments under `src/main_tests/runtime_tests/` while preserving top-level selector names.
3. Update `runtime_tests.rs` to a bounded shell (imports, shared helpers, include declarations).
4. Add ownership-boundary note in `docs/foundation/runtime-watchdog-attestation.md`.
5. Run targeted and regression test commands.

## Affected Modules
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/*.rs`
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `docs/foundation/runtime-watchdog-attestation.md`

## Risks and Mitigations
- Risk: selector drift from module refactors breaks scripts/docs.
  - Mitigation: use `include!()` at module root so function paths remain unchanged.
- Risk: accidental helper visibility/regression across split files.
  - Mitigation: keep shared imports/helpers in `runtime_tests.rs` shell and run targeted runtime tests.
- Risk: shell budget assertions become flaky.
  - Mitigation: assert deterministic structural markers (include declarations + no inline tests) with conservative line bounds.

## Contracts and Interfaces
- `runtime_tests.rs` remains module root for `main_tests::runtime_tests::*` selectors.
- include fragment files are internal implementation detail under `src/main_tests/runtime_tests/`.
- contract test markers enforce decomposition and bounded-shell shape.

## Verification Strategy
- RED: run new extraction contract test before decomposition (expect fail).
- GREEN: split runtime tests into include fragments and rerun targeted contract + runtime selectors.
- REGRESSION: run full `main_module_extraction_contract` and formatting/lint checks.
