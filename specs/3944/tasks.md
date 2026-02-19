# Issue #3944 Tasks

- Issue: #3944
- Status: Completed

## Ordered Tasks
- [x] T1 (Red): add runtime test-shell extraction assertions to `main_module_extraction_contract.rs` and capture failing run.
- [x] T2 (Green): extract runtime test bodies from `runtime_tests.rs` into focused include fragments under `src/main_tests/runtime_tests/`.
- [x] T3 (Green): wire `runtime_tests.rs` as bounded shell with deterministic include ownership markers.
- [x] T4 (Regression): run targeted runtime selector tests and full `main_module_extraction_contract` suite.
- [x] T5 (Verify): update runtime ownership docs and verify spec-to-test mapping evidence.

## Tier Mapping
- Unit: runtime shell structural assertions in extraction contract test.
- Functional: runtime test shell delegates to focused include fragments.
- Integration: representative `main_tests::runtime_tests::*` selectors still execute with stable names.
- Regression: extraction contract prevents inline-monolith reintroduction.
- Performance: N/A (test-structure refactor only).
