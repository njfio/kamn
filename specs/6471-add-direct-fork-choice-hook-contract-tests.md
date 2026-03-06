# Spec: Issue 6471 - Add direct fork-choice hook contract tests

## Objective
Add direct contract coverage for `DeterministicCompetingBranchForkChoiceHook` so
its core acceptance/rejection rules are verified independently of broader
transport/block-pipeline scenarios.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/block_pipeline/fork_choice.rs`
- Outputs:
  - New dedicated integration test file for direct fork-choice hook behavior.
  - Coverage for seeded head initialization, stale/duplicate rejection,
    higher-height acceptance, lexicographic tie-break acceptance/rejection, and
    canonical-head mutation rules.

## Boundaries/Non-goals
- No fork-choice behavior changes.
- No block-pipeline API changes.
- No commit-store work in this issue.

## Failure modes
- Duplicate candidates are not rejected deterministically.
- Stale candidates update canonical head when they should not.
- Higher-height or lexicographically lower equal-height candidates fail to
  replace canonical head.
- Reject paths mutate canonical head unexpectedly.

## Acceptance criteria (testable booleans)
- [x] AC-1: Direct tests exist for `DeterministicCompetingBranchForkChoiceHook`.
- [x] AC-2: Empty-head evaluation accepts first candidate and seeds canonical
      head.
- [x] AC-3: Higher block height replaces canonical head.
- [x] AC-4: Lower block height rejects with
      `fork_choice_stale_block_height` and preserves head.
- [x] AC-5: Equal height + equal digest rejects with
      `fork_choice_duplicate_candidate` and preserves head.
- [x] AC-6: Equal height + lexicographically lower digest accepts and replaces
      head.
- [x] AC-7: Equal height + lexicographically higher digest rejects with
      `fork_choice_tie_break_loser` and preserves head.
- [x] AC-8: `cargo test -p kamn-core --test block_pipeline_fork_choice` passes.

## Files to touch
- `specs/6471-add-direct-fork-choice-hook-contract-tests.md`
- `crates/kamn-core/tests/block_pipeline_fork_choice.rs`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error semantics
- Preserve existing deterministic rejection reason codes.
- Preserve current hook state-transition behavior.

## Test plan
- Red:
  - Add dedicated hook tests for all acceptance and rejection paths.
  - Verify the new test target fails before implementation wiring.
- Green:
  - Add minimal direct coverage using current public types and APIs.
- Refactor:
  - Keep fixtures/helpers concise and deterministic.
- Integration:
  - `cargo test -p kamn-core --test block_pipeline_fork_choice`

## Phase 6 integration evidence
- No new production entrypoint wiring was required because this issue adds direct
  test coverage only.
- Real-path integration remains exercised by
  `cargo test -p kamn-core --test block_pipeline_transport_fed -- --nocapture`,
  including fork-choice acceptance and rejection through the transport-fed block
  pipeline.
- Direct coverage target passes via
  `cargo test -p kamn-core --test block_pipeline_fork_choice -- --nocapture`.

## Deviations
- Refreshed `fixtures/ci/test_file_size_policy_baseline.env` because the new
  direct and contract test targets increased the repository test-file inventory
  from 429 to 431 without changing any size-budget counts.
