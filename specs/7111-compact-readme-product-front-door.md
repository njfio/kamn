# Issue 7111: Compact README Product Front Door

## Objective

Make the root README a concise, human-first entrypoint for KAMN's real MVP while
keeping the evaluator path, honest claim boundaries, local proof command, and
agent-maintainer handoff intact within the enforced 200-line cap.

## Inputs And Outputs

Input:

- The current 240-line root README.
- Existing evaluator and contract-detail docs.
- The shipped canonical commands and verifier.

Output:

- A root README of at most 200 lines.
- One prominent canonical three-agent devnet transaction path.
- Links to detailed setup and reference material instead of duplicated blocks.

## Boundaries And Non-Goals

- Do not change commands, runtime behavior, proof semantics, claim labels, or
  setup requirements.
- Do not add claims beyond the merged #7103 evidence.
- Do not copy secrets, keypairs, funded addresses, or local artifact values.
- Do not expand the evaluator runbook or contract-reference document.
- Preserve human orientation before maintainer/agent process detail.

## Failure Modes

- README exceeds 200 lines.
- `make demo-agent-transaction`, `make demo-mvp`, or the standalone verifier is
  missing or presented with the wrong trust boundary.
- Devnet settlement is described as local simulation or production value.
- Local-only, dry-run, placeholder, or roadmap behavior is counted as success.
- The architecture index, evaluator runbook, contract reference, or `AGENTS.md`
  handoff disappears.
- Detailed environment setup is duplicated instead of linked.

## Error Semantics

Documentation contracts fail directly when required markers or the line cap
drift. No alias comments or hidden markers should substitute for clear headings.

## Acceptance Criteria

- [x] README is 200 lines or fewer.
- [x] Opening explains what KAMN is, why it exists, and current MVP scope.
- [x] Canonical evaluator path is `make demo-agent-transaction`.
- [x] Local bounded proof and standalone verifier commands remain visible.
- [x] Settlement success remains explicitly Solana devnet-backed.
- [x] Non-claims and all six claim labels remain clear.
- [x] Detailed setup links to `docs/validation/mvp-evaluator-demo.md`.
- [x] Architecture, contract-reference, and `AGENTS.md` links remain visible.
- [ ] README contract tests, formatting, strict clippy, `make check`, and
      `make test` pass.

## Files To Touch

- `README.md`
- This spec only.

## Test Plan

RED is the existing committed contract:

```text
README.md must stay <= 200 lines for onboarding; found 240
```

GREEN verification:

```bash
CARGO_TARGET_DIR=target/mvp-demo-proof cargo test -p kamn-core \
  --test readme_compact_contract
cargo fmt --check
CARGO_TARGET_DIR=target/mvp-demo-proof cargo clippy \
  --workspace --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=target/mvp-demo-proof make check
CARGO_TARGET_DIR=target/mvp-demo-proof make test
```

## Completion Evidence

- README is 147 lines and both README contract binaries pass all 6 tests.
- `cargo fmt --check`, strict workspace clippy, and `make check` pass.
- Full `make test` passes both README binaries, then fails the unrelated
  `test_file_size_policy` inventory baseline (`1317` actual versus `1288`).
