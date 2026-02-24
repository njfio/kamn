# Plan: Issue #5842

## Approach
1. Land lifecycle artifacts for #5842 and wire implementation scope to existing review contract lanes (no new test-file proliferation).
2. Introduce deterministic review freeze enforcement for R51+ docs in `review_r53_docs_contract.rs` using explicit baseline fingerprints.
3. Add/validate R56 unresolved-item markers and corrected attribution semantics in review docs plus docs-contract assertions.
4. Upgrade tracked spec-dir counting helper(s) to explicit git-tree semantics and retain contamination regression tests.
5. Keep production `expect()` inventory semantics deterministic and enforce marker consistency with computed scope.
6. Reduce shell LOC by compacting high-noise script surfaces without changing command-surface contract behavior.
7. Run targeted RED/GREEN verification commands and package PR evidence per AGENTS template.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `docs/review/gaps-and-issues-r56.md`
- `docs/review/README.md`
- `docs/review/*.policy` (freeze/governance marker policy as needed)
- `scripts/ci/test_ci_tools.sh`
- `scripts/ci/check_no_production_expect.py` (only if further scope correction is required)

## Risks and Mitigations
- Risk: freeze enforcement can over-constrain legitimate same-PR authoring updates.
  - Mitigation: freeze only after establishing deterministic baseline in this PR; enforce for subsequent changes.
- Risk: shell script compaction can break ordering-sensitive CI behavior.
  - Mitigation: preserve command order; validate with `bash -n` and targeted CI-tool contract tests.
- Risk: marker assertions can encode stale narrative claims.
  - Mitigation: compute status from ratios/counts and assert derived expectations.

## Interface/Contract Changes
- Review-contract surface gains explicit R56 unresolved-state + freeze-enforcement invariants.
- Tracked-only spec-dir counting semantics clarified to git-tree based enumeration.
- Shell-surface closure evidence explicitly captured as measurable delta markers.

## ADR
- Not required (no dependency/protocol architecture change; governance/docs-contract hardening only).
