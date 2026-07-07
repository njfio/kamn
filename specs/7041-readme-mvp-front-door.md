# 7041 README MVP Front Door

## Objective

Make the root README the current KAMN front door for human evaluators, developers,
and AI agents. It should explain what KAMN is, what the repository can prove
today, how to run the MVP demo, how devnet-backed settlement evidence is claimed,
and where deeper implementation and process references live.

## Inputs/Outputs

Inputs:
- Existing root `README.md`
- Current evaluator runbook: `docs/validation/mvp-evaluator-demo.md`
- Current issue/process contract: `AGENTS.md`
- Current make targets: `make demo-mvp`, `make check`, `make test`,
  `make ci-tools`

Outputs:
- Updated root `README.md`
- A README contract test that protects MVP front-door ordering and claim-boundary
  content

## Boundaries/Non-goals

- Do not change runtime, demo, settlement, verifier, or devnet behavior.
- Do not add protocol, production, mainnet, custody, generalized exchange, or
  real-economic-value claims.
- Do not duplicate the full evaluator runbook; link to it for depth.
- Do not commit local proof artifacts, keypairs, generated package metadata, or
  unrelated local files.
- Do not weaken tests, clippy, formatting, or proof semantics.

## Failure Modes

- README buries the MVP demo below internal implementation details.
- README implies production readiness, mainnet support, or real economic value.
- README mentions settlement or asset movement without the `devnet-backed`
  boundary.
- README omits the verifier command, claim boundaries, or evaluator runbook link.
- README stops serving AI agents and developers by removing repo map, workflow,
  validation, or architecture links.

## Acceptance Criteria

- [x] README opens with human-friendly what/where/why/current-capability content.
- [x] README includes `make demo-mvp` and the verifier command near the top.
- [x] README explicitly separates local-only proof, devnet-backed proof,
  roadmap/not-claimed work, and production-readiness boundaries.
- [x] README links to `docs/validation/mvp-evaluator-demo.md`.
- [x] README preserves developer and AI-agent depth sections for repo map,
  workflow/process, validation gates, architecture, and contract references.
- [x] A docs contract test fails against the old README and passes against the
  updated README.

## Files to Touch

Likely:
- `README.md`
- `crates/kamn-e2e-harness/tests/readme_mvp_front_door_contract.rs`

Only if needed:
- `crates/kamn-e2e-harness/Cargo.toml`

## Error Semantics

- The docs contract test should hard-fail with a clear message if required README
  sections, commands, links, or claim-boundary language are missing or ordered
  incorrectly.
- The README must use explicit claim labels and avoid ambiguous language around
  value movement.

## Test Plan

Red:
- Add a README contract test that requires the MVP demo/proof story to appear
  before deep implementation references and requires the current claim boundaries.
- Confirm it fails against the existing README.

Green:
- Rewrite the root README to satisfy the contract while preserving current
  developer/agent reference depth.

Refactor:
- Keep the README concise at the top, move deeper references below the quickstart,
  and avoid duplicated runbook content.

Verification:
- `cargo fmt --check`
- `cargo test -p kamn-e2e-harness --test readme_mvp_front_door_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness --test mvp_demo_command_contract --test mvp_demo_claim_contract -- --nocapture`
