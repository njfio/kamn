# Issue 6653: Normalize Phase 6 Evidence And Spec Index Coverage

## Objective

Define one canonical Phase 6 evidence format for closure-ready top-level issue specs, fail closed on legacy heading drift for newly updated closure-ready specs, and replace the curated-only specs index with a full-corpus index mechanism that covers every top-level `specs/*.md` issue spec.

## Inputs/Outputs

### Inputs
- Existing top-level issue specs under `specs/*.md`
- Existing Phase 6 evidence policy doc and checker
- Existing `specs/INDEX.md` curated-track overview

### Outputs
- Canonical Phase 6 evidence policy markers and remediation guidance
- Fail-closed checker/tests for canonical heading plus execution markers
- Full-corpus spec index overview plus sharded index documents covering every top-level issue spec
- Contract/tests that verify index coverage and policy shape

## Boundaries/Non-goals

- Do not backfill every historical spec in this issue
- Do not rewrite nested archival planning docs under `specs/**`
- Do not change the seven-phase workflow itself
- Treat top-level `specs/*.md` issue specs as the authoritative closure-ready corpus; nested `specs/<id>/*.md` planning artifacts stay out of the index coverage contract

## Failure Modes

- Closure-ready spec uses a non-canonical Phase 6 heading and bypasses normalization
- Closure-ready spec has the canonical heading but omits executable command evidence
- Specs index claims full coverage but omits one or more top-level issue specs
- Specs index includes entries outside the authoritative top-level issue-spec corpus
- Docs drift from the checker behavior and give contributors the wrong closure instructions

## Acceptance Criteria

- [ ] AC-1: Canonical Phase 6 evidence format is documented as `## Phase 6 integration evidence` with `Executed:` and backticked commands.
- [ ] AC-2: The policy checker fails closed when a closure-ready top-level spec uses a legacy Phase 6 heading variant instead of the canonical heading.
- [ ] AC-3: The policy docs include a migration/backfill plan for legacy specs that still use older headings or are missing Phase 6 evidence entirely.
- [ ] AC-4: `specs/INDEX.md` becomes a full-corpus entrypoint that points to sharded index files covering every top-level `specs/*.md` issue spec exactly once.
- [ ] AC-5: Automated contracts verify both the canonical Phase 6 evidence policy and full-corpus index coverage.

## Files To Touch

- `specs/6653-normalize-phase6-evidence-and-spec-index-coverage.md`
- `docs/planning/spec-phase6-evidence-policy.md`
- `scripts/ci/check_spec_phase6_evidence_policy.sh`
- `scripts/ci/test_check_spec_phase6_evidence_policy.sh`
- `specs/INDEX.md`
- `specs/index/6000-6499.md`
- `specs/index/6500-6999.md`
- `scripts/ci/check_specs_index_coverage.sh`
- `scripts/ci/test_check_specs_index_coverage.sh`
- `crates/kamn-core/tests/specs_index_docs.rs`

## Error Semantics

- Policy and index checkers fail closed with deterministic reason codes
- Invalid or incomplete Phase 6 evidence returns explicit `spec_phase6_*` reason markers
- Missing or stale index coverage returns explicit `specs_index_*` reason markers
- No silent normalization or fallback to legacy headings

## Test Plan

1. Add red shell-contract coverage proving the Phase 6 checker rejects legacy headings on closure-ready specs and that the index coverage checker rejects missing or extra entries.
2. Update docs and checkers minimally until the shell contracts pass.
3. Update Rust docs contracts so the index and policy markers are asserted from compiled tests.
4. Run targeted shell and Rust tests for the policy and index contracts.
5. Record Phase 6 integration evidence in this spec after the real checkers and contracts pass.
