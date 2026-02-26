# Plan: Issue #6045

## Approach
1. Add a dedicated CI checker wrapper for production-target `expect()` enforcement via clippy (`--lib --bins`).
2. Add a deterministic shell contract test that validates command arguments and fail-closed behavior without requiring a full workspace compile.
3. Wire the new checker into:
   - `scripts/ci/test_ci_tools.sh` fast-mode command surface.
   - `.github/workflows/ci-fast-gate.yml` Rust lane.
4. Update CI strategy docs and corresponding docs-contract assertions for the new production-target scope marker.
5. Run targeted verification on changed contract/workflow/doc tests.

## Affected Modules
- `scripts/ci/check_no_production_expect_clippy.sh` (new)
- `scripts/ci/test_check_no_production_expect_clippy.sh` (new)
- `scripts/ci/test_ci_tools.sh`
- `.github/workflows/ci-fast-gate.yml`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks / Mitigations
- Risk: workflow command drift may bypass production-target scope.
  Mitigation: add explicit contract test asserting `--lib --bins` presence and test-target absence.
- Risk: documentation and workflow drift.
  Mitigation: update docs + doc-contract tests in the same change set.
- Risk: checker command changes break local contributor flows.
  Mitigation: keep checker as a small standalone wrapper with deterministic output and no side effects.

## Interfaces / Contracts
- CI contract command:
  - `cargo clippy --workspace --lib --bins -- -D warnings -D clippy::expect_used`
- Workflow contract:
  - `ci-fast-gate` must run production-target checker when Rust lane is enabled.
- Documentation contract:
  - Strategy doc includes checker command and explicit production target marker.
