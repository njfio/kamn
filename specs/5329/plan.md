# Issue #5329 Plan

## Approach
1. Audit current ignored-test inventory from source to identify long-lived candidates with low-risk promotion path.
2. Promote two deep-lane tests by removing `#[ignore]` and adding deterministic opt-in env gating (`KAMN_KOLME_LOCAL_HEAVY=1`).
3. Regenerate and update ignored-test baseline and metadata fixtures to match source truth.
4. Add explicit `disposition` notes in metadata for each remaining ignored entry.
5. Run ignored-test drift/parser checks and targeted Rust tests for promoted functions.

## Affected Modules
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/message_lifecycle.rs`
- `fixtures/ci/ignored_test_inventory_baseline.json`
- `fixtures/ci/ignored_test_inventory_metadata.json`
- `specs/5329/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: promoted deep-lane tests add CI runtime overhead.
  - Mitigation: return early unless explicit local-heavy env flag is set.
- Risk: baseline/metadata drift breaks contract lanes.
  - Mitigation: regenerate inventory and run both drift + parser contract tests before commit.

## Interfaces and Contracts
- No production API changes.
- CI ignored-test inventory schema remains unchanged.
- Metadata enhancements (`disposition`) are additive and backward-compatible.
