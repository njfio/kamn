# Issue #3785 Tasks

- Issue: #3785
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing docs-contract assertion for unified API-observability local-heavy CI exclusion command markers.
- [x] T2 (Green): update strategy docs heavy-integration contract section with unified lane exclusion command and fail-closed rule.
- [x] T3 (Functional): run `scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`.
- [x] T4 (Regression): run targeted docs-contract test + fmt + clippy + shell guardrails.
- [ ] T5 (Verify): open/merge PR and close issue with DoD shell-surface metrics.

## Tier Mapping
- Unit: N/A (selector predicate is covered in existing shell functional contract; no new unit helper introduced).
- Functional: `scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`
- Integration: `cargo test -p kamn-core --test ci_strategy_docs`
- Regression: targeted exact test + lint + shell guardrails
