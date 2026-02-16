# Plan: Issue #4454

Status: Completed
Issue: #4454

## Approach

1. Add RED assertions in `scripts/ci/test_check_no_production_expect.sh` for:
   - production `panic!` fixture
   - production `unreachable!` fixture
   - production unsafe env fallback-default fixture
2. Extend `scripts/ci/check_no_production_expect.py` detection logic to satisfy RED tests while
   preserving `#[cfg(test)]` exclusions.
3. Create/update `docs/security/secure-coding.md` with panic reachability + unsafe fallback
   failure-case policy markers.
4. Add docs contract test `crates/kamn-core/tests/secure_coding_docs.rs`.
5. Run RED/GREEN loop and scoped hygiene checks.

## Affected Modules

- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `docs/security/secure-coding.md` (new)
- `crates/kamn-core/tests/secure_coding_docs.rs` (new)
- `specs/4454/*`

## Risks and Mitigations

- Risk: false positives in fallback detection.
  - Mitigation: target deterministic unsafe-fallback patterns in tests; keep test-only exclusions.
- Risk: checker scope drift affecting unrelated test files.
  - Mitigation: preserve existing exclusion rules for test modules/files.

## Interfaces / Contracts

- Checker contract remains deterministic text output with `status`, `violation_count`, and
  `violation=<file:line:snippet>` entries.
- Docs contract adds explicit secure-coding markers for panic-path and unsafe fallback policy.

## ADR

Not required: no architecture or dependency change.
