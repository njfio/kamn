# Spec: Issue 6353 - Extract CLI Kolme Live option parsing into module

## Objective
Move Kolme Live option parsing out of `crates/kamn-node/src/cli.rs` into a dedicated module so `parse_args` remains focused on top-level argument dispatch.

## Inputs/Outputs
- Inputs:
  - Inline Kolme Live match arms in `cli.rs`:
    - `--kolme-live-base-url`
    - `--kolme-live-provider-hint`
    - `--kolme-live-signing-profile`
    - `--kolme-live-strict-signer-contracts`
    - `--kolme-live-signer-profile`
    - `--kolme-live-signer-key-source`
- Outputs:
  - New Kolme Live option parsing module.
  - `cli.rs` wiring updates to delegate Kolme Live option parsing.
  - Contract assertions for Kolme Live module declaration and inline-arm removal.

## Boundaries/Non-goals
- Do not change CLI flags, defaults, or parse order.
- Do not change runtime-mode validation semantics.
- Do not change daemon or endpoint option parsing semantics.
- Do not add dependencies.

## Failure modes
- Missing module wiring causes compile/test failures.
- Kolme Live parse behavior drift changes required-value error outcomes.
- Inline Kolme Live match arms remain in `cli.rs` and violate extraction contract.

## Acceptance criteria (testable booleans)
- [x] `cli.rs` declares a dedicated Kolme Live option parser module.
- [x] `cli.rs` no longer contains inline Kolme Live option match arms.
- [x] Kolme Live option parse behavior and `ConfigError` outcomes remain unchanged.
- [x] Extraction contract includes Kolme Live module assertions and passes.
- [x] Existing CLI tests remain green.

## Files to touch
- `crates/kamn-node/src/cli.rs`
- `crates/kamn-node/src/cli_kolme_live_option_parsing.rs` (new)
- `crates/kamn-node/tests/cli_module_extraction_contract.rs`
- `specs/6353-cli-kolme-live-option-parser-module-extraction.md`

## Error semantics
- Preserve hard-fail behavior:
  - missing Kolme Live option values continue returning `ConfigError::MissingArgumentValue(<flag>)`
  - boolean strict-signer option remains a presence flag with no value.

## Test plan
- Red:
  - Extend `cli_module_extraction_contract` with Kolme Live module declaration and inline-arm removal assertions; confirm failure.
- Green/refactor/integration:
  - `cargo test -p kamn-node --test cli_module_extraction_contract`
  - `cargo test -p kamn-node cli_tests`
  - `cargo test -p kamn-node --test main_tests_command_surface_parity_contract`

## Phase 6 integration evidence
- `cargo test -p kamn-node --test cli_module_extraction_contract` (pass)
- `cargo test -p kamn-node cli_tests` (pass)
- `cargo test -p kamn-node --test main_tests_command_surface_parity_contract` (pass)
- `timeout 30s cargo run -p kamn-node -- --role processor --runtime-mode planning --expected-state-hash reason --proposal 'alpha|beta|1|reason' --kolme-live-base-url https://example.invalid --kolme-live-provider-hint in-memory --kolme-live-signing-profile localnet-kolme-live-signing --kolme-live-strict-signer-contracts --kolme-live-signer-profile managed --kolme-live-signer-key-source env --output text --diagnostics basic` (pass)
- `timeout 30s cargo run -p kamn-node -- --role processor --runtime-mode planning --expected-state-hash reason --proposal 'alpha|beta|1|reason' --output text --diagnostics basic --kolme-live-signer-key-source` (expected hard-fail: missing `--kolme-live-signer-key-source`)

## Deviations
- None.
