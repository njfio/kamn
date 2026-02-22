# Issue #5566 Plan - PRD Phase-4b Harness Run/Verify Command Contracts

## Approach
1. Add RED tests for:
   - command parser success/failure paths (`run`, `verify`)
   - scenario CSV parsing/validation against `S-01..S-15`
   - verify output-file behavior + deterministic report content
2. Implement command contract types and parser helpers in `kamn-e2e-harness` library module.
3. Update binary `main.rs` to dispatch `run` and `verify` commands.
4. Implement verify output writing using existing deterministic report generator.
5. Add docs/research phase-4b markers and milestone index updates.
6. Run gates and targeted regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/main.rs`
- `crates/kamn-e2e-harness/src/verify.rs` (if required for command helper integration)
- `crates/kamn-e2e-harness/tests/*` (new phase-4b command contract tests)
- `docs/research/e2e-live-testing-prd-phase4b-gap-analysis.md`
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: parser ambiguity and brittle argument ordering.
  - Mitigation: explicit command enum + deterministic positional/flag validation tests.
- Risk: filesystem side effects in tests.
  - Mitigation: use temporary directories under `std::env::temp_dir()` and cleanup files.
- Risk: drift from PRD command examples.
  - Mitigation: use exact flag names from PRD sections 9/12 and lock with conformance tests.

## Interfaces / Contracts
- Command parser contract:
  - `run --mode <mode> --evidence-dir <path> --scenarios <csv>`
  - `verify --evidence-dir <path> --kolme-chain-dump <path> --output <path>`
- Scenario selection contract:
  - IDs must exist in registry `S-01..S-15`
- Verify output contract:
  - deterministic report JSON including `schema_check`, `proof_check`, `chain_check`, `content_check`

## ADR
- Not required (command surface completion for existing harness architecture).
