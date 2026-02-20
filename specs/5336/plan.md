# Issue #5336 Plan

## Implementation Approach
1. Signer test lock hardening:
- Replace local `managed_backend_env_lock` static lock with `crate::signer_test_env_lock()` alias in managed-backend test module.
- Add explicit regression proving lock alias identity by pointer equality.

2. Branch hygiene closure evidence:
- Measure current remote branch count.
- Capture recent `branch-cleanup` workflow run evidence.
- Update `docs/review/gaps-and-issues-r45.md` status narrative and priority summary markers to reflect measured post-cleanup state.

3. Docs-contract wave-4 tranche:
- Add `crates/kamn-core/tests/docs_contract_wave4_harness.rs`.
- Migrate 11 low-coupling include_str suites into harness modules with assertion parity.
- Remove migrated per-doc files.
- Re-measure include_str file count and verify harness tests.

## Affected Modules
- `crates/kamn-node/src/signer/managed_backend.rs` (test-only)
- `crates/kamn-core/tests/docs_contract_wave4_harness.rs` (new)
- migrated docs-contract files under `crates/kamn-core/tests/`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: accidental assertion drift during harness migration.
  - Mitigation: copy assertions verbatim into per-module harness sections; run targeted harness tests.
- Risk: flaky signer stress not reproduced locally.
  - Mitigation: run targeted parallel commands with `--test-threads=16` and preserve regression lock-domain test.
- Risk: branch evidence becomes stale quickly.
  - Mitigation: capture fresh measurements in same change set.

## Interfaces / Contracts
- No production API changes.
- Test contract added: managed-backend tests must alias shared signer env lock domain.

## ADR
- Not required: no new dependency, protocol, or architecture-level runtime behavior change.
