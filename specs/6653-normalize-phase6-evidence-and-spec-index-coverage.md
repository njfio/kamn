# Issue 6653: Normalize Phase 6 Evidence And Spec Index Coverage

- Status: Implemented

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

- [x] AC-1: Canonical Phase 6 evidence format is documented as `## Phase 6 integration evidence` with `Executed:` and backticked commands.
- [x] AC-2: The policy checker fails closed when a closure-ready top-level spec uses a legacy Phase 6 heading variant instead of the canonical heading.
- [x] AC-3: The policy docs include a migration/backfill plan for legacy specs that still use older headings or are missing Phase 6 evidence entirely.
- [x] AC-4: `specs/INDEX.md` becomes a full-corpus entrypoint that points to sharded index files covering every top-level `specs/*.md` issue spec exactly once.
- [x] AC-5: Automated contracts verify both the canonical Phase 6 evidence policy and full-corpus index coverage.

## Files To Touch

- `specs/6653-normalize-phase6-evidence-and-spec-index-coverage.md`
- `docs/planning/spec-phase6-evidence-policy.md`
- `scripts/ci/check_spec_phase6_evidence_policy.sh`
- `scripts/ci/test_check_spec_phase6_evidence_policy.sh`
- `.ci/shell_test_surface_ratio_thresholds.env`
- `.ci/shell_test_surface_ratio_waiver_6653.env`
- `specs/INDEX.md`
- `specs/index/6000-6499.md`
- `specs/index/6500-6999.md`
- `scripts/ci/check_specs_index_coverage.sh`
- `scripts/ci/test_check_specs_index_coverage.sh`
- `.github/CONTRIBUTING.md`
- `scripts/ci/test_ci_tools.sh`
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

## Refactor Evidence

- `scripts/ci/check_spec_phase6_evidence_policy.sh` remains below the 200 LOC file limit at `189` lines.
- `scripts/ci/check_specs_index_coverage.sh` remains below the 200 LOC file limit at `172` lines.
- `scripts/ci/test_check_spec_phase6_evidence_policy.sh` remains below the 200 LOC file limit at `193` lines.
- `scripts/ci/test_check_specs_index_coverage.sh` remains below the 200 LOC file limit at `118` lines after extracting shared failure helpers.
- The specs index is split into two shard files to keep each index artifact below the 200 LOC file limit.

## Phase 6 integration evidence

- Executed:
  - `bash scripts/ci/test_check_spec_phase6_evidence_policy.sh`
  - `bash scripts/ci/test_check_specs_index_coverage.sh`
  - `cargo test -p kamn-core --test specs_index_docs -- --nocapture`
  - `bash scripts/ci/check_spec_phase6_evidence_policy.sh --output-json /tmp/spec-phase6-evidence-policy-report.json`
  - `bash scripts/ci/check_specs_index_coverage.sh --output-json /tmp/specs-index-coverage.json`
  - `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
- Results:
  - Canonical Phase 6 evidence policy shell contract passed.
  - Specs index coverage shell contract passed.
  - `specs/INDEX.md` docs contract passed.
  - Both real checkers returned `status=ok` and `final_decision=GO`.
  - Shell-test surface ratio policy passed after rotating the waiver pointer to `.ci/shell_test_surface_ratio_waiver_6653.env` with `max_shell_test_file_delta=3`.
  - The fast-mode CI-tools entrypoint executed the new specs-index contract lane successfully and later hit the known unrelated runtime-suite interaction in `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` (`request-validation probe expected 400 status; got 401`).

## Deviations

- The repo currently has no closure-ready top-level issue specs before this issue (`closure_ready_spec_count=0`), so the stricter canonical-heading enforcement landed without requiring a historical top-level backfill wave.
- Full `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` completion remains blocked by the unrelated late axum-ingress runtime-suite `400` vs `401` interaction. The new specs-index coverage lane was already exercised successfully earlier in the same real entrypoint run, so that late failure was documented rather than treated as an overlap for `#6653`.
