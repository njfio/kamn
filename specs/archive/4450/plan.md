# Plan: Issue #4450

Status: Completed
Issue: #4450

## Approach

1. Extend runtime extraction source contract coverage in
   `crates/kamn-node/tests/main_module_extraction_contract.rs` with explicit
   module-boundary parity assertions.
2. Extend runtime docs contract coverage in
   `crates/kamn-core/tests/runtime_architecture_docs.rs` for module-boundary parity
   taxonomy and guard-command markers.
3. Update `docs/architecture/runtime.md` with explicit runtime module-boundary parity
   drift markers and deterministic validation commands.
4. Execute RED -> GREEN loop with targeted tests, then run scoped formatting/lint checks.

## Affected Modules

- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `crates/kamn-core/tests/runtime_architecture_docs.rs`
- `docs/architecture/runtime.md`
- `specs/4450/*`

## Risks and Mitigations

- Risk: string-based docs/source contract assertions can become brittle.
  - Mitigation: assert stable, intentional marker strings only; avoid over-broad matching.
- Risk: accidental behavior changes in runtime code.
  - Mitigation: constrain changes to tests/docs only for this subtask.

## Interfaces / Contracts

- Runtime extraction boundary ownership contract:
  - `main.rs` orchestration only
  - extracted runtime boundaries in `runtime_orchestration.rs` and `runtime_kolme_live.rs`
- Runtime docs parity contract:
  - deterministic taxonomy marker version
  - deterministic reason codes
  - deterministic command-surface markers

## ADR

Not required: no dependency or architecture decision changes, only contract guardrail
coverage and docs parity hardening.
