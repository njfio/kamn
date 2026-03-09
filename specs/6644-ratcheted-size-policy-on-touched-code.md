# 6644 - Ratcheted file/function size policy on touched code

## Objective

Add a fail-closed ratcheted size-policy gate for touched Rust code so newly oversized files and functions are blocked without freezing the existing legacy baseline. Preserve visibility into current debt with a committed baseline inventory.

## Inputs/Outputs

### Inputs
- Git diff between `HEAD` and the merge-base of the configured base ref
- Rust source files under `crates/**/*.rs`
- Policy thresholds for maximum file lines and maximum function body lines
- Baseline inventory fixture capturing current oversized Rust files/functions
- Fast Gate workflow wiring and local CI-tool regression scripts

### Outputs
- A CI checker report JSON listing touched oversized file/function offenders and policy status
- A committed baseline inventory fixture for current oversized Rust files/functions
- Contract/regression tests covering ratchet behavior, baseline schema, and workflow wiring
- Docs/strategy markers describing the new touched-code size ratchet

## Boundaries/Non-goals

- Do not hard-fail untouched legacy oversized files/functions in this issue
- Do not remediate existing oversized files/functions in this issue
- Do not add non-stdlib parser dependencies
- Limit scope to Rust source files under `crates/**`; test-file inventory policy remains in place separately

## Failure Modes

- Threshold fixture missing, malformed, or schema-mismatched
- Baseline inventory missing, malformed, or schema-mismatched
- Git base/merge-base cannot be resolved for touched-file comparison
- A touched Rust file exceeds the file line-count limit and its base version was compliant or absent
- A touched Rust function exceeds the function line-count limit and its base version was compliant or absent
- Checker report JSON cannot be written
- Workflow/docs/tests drift so the gate is not wired into real Fast Gate entrypoints

## Acceptance Criteria

- [ ] A dedicated checker evaluates touched Rust files/functions against size limits using git-base comparison
- [ ] The checker fails when a changed file exceeds the file-size policy and its base version was compliant or absent
- [ ] The checker fails when a changed function exceeds the function-size policy and its base version was compliant or absent
- [ ] Existing oversized Rust files/functions are captured in a committed baseline inventory fixture with schema version
- [ ] Checker output identifies exact offending file/function paths and measured line counts
- [ ] Fast Gate executes the checker and uploads its report artifact
- [ ] Local CI tool regression coverage exercises pass/fail/error-path behavior for the checker and workflow wiring
- [ ] CI docs/contracts mention the touched-code size ratchet markers and command surface

## Files To Touch

- `specs/6644-ratcheted-size-policy-on-touched-code.md`
- `scripts/ci/check_touched_rust_size_policy.py`
- `scripts/ci/check_touched_rust_size_policy.sh`
- `scripts/ci/test_check_touched_rust_size_policy.sh`
- `scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `scripts/ci/test_ci_tools.sh`
- `scripts/lib/exec_registry.json`
- `.github/workflows/ci-fast-gate.yml`
- `fixtures/ci/touched_rust_size_policy_thresholds.json`
- `fixtures/ci/touched_rust_size_policy_baseline.json`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Error Semantics

- The checker fails closed on missing files, invalid schema, invalid threshold ordering, unresolved git base, unreadable source, parse mismatches, or report-write failures
- Checker stdout remains deterministic key/value status markers plus structured reason codes
- The JSON report includes schema version, status, policy decision, merge-base commit, touched file list, and offender details
- Entrypoint scripts report errors once; interior helpers return structured failures

## Test Plan

1. Red: add a shell regression harness for the checker covering:
   - compliant touched file/function
   - new oversized file violation
   - new oversized function violation
   - oversized legacy file/function allowed when already oversized in base fixture input
   - missing/invalid threshold or baseline fixture
   - unresolved git base / invalid report path
2. Red: extend Fast Gate wiring regression tests so they fail until the workflow invokes the checker and uploads telemetry
3. Red: extend CI strategy docs contracts to require the new checker markers and remediation language
4. Green: implement the checker, wrapper wiring, fixtures, and workflow step
5. Refactor: extract parser/report helpers so functions stay within AGENTS limits and failure semantics stay deterministic
6. Integration: run targeted shell tests, docs contract tests, and the Fast Gate CI tools harness that exercises the real entrypoints

## Notes / Deviations

- Function-size comparison will use a fail-closed lexical Rust function-span scanner based on balanced braces and normalized function headers rather than a new AST dependency.
- The committed baseline inventory is for visibility and drift review; the touched-code ratchet will use git-base comparison as the enforcement authority.
