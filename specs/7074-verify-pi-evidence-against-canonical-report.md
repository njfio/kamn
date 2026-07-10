# Issue 7074: Verify Pi Evidence Against One Canonical Report

## Objective

Make the evaluator's Pi workflow reproducible by allowing the canonical MVP
verifier to validate a Pi agent-harness evidence artifact directly against the
exact canonical report that produced its actor and observation receipts.

## Inputs/Outputs

- Input: `verify-mvp-demo --report <path>` for the immutable canonical report.
- Input: optional `--agent-harness-evidence <path>` for Pi extension evidence.
- Input: canonical three-agent claim, view, actor receipt, and observation
  receipt fields already present in a devnet-backed report.
- Output: `PASS` only when the report verifies and the supplied Pi artifact
  agrees with the report path, claim boundaries, actor tool receipts, and
  canonical observation receipt artifact/digest references.
- Output: the verifier result identifies the validated evidence path when the
  optional artifact is supplied.

## Boundaries/Non-goals

- Do not mutate, patch, or relabel the canonical report.
- Do not rerun settlement or submit a second devnet transaction.
- Do not claim Pi performs settlement or asset movement.
- Keep Pi agent-harness verification `local-only`; value movement remains
  `devnet-backed` only through the canonical report evidence.
- Do not add dependencies, agent runtimes, wallet formats, or settlement paths.
- Do not generalize this MVP handshake into production attestation or mainnet.

## Failure Modes

- The supplied evidence names a different report path.
- The supplied evidence omits an actor tool receipt required for the report.
- The supplied evidence omits a canonical observation receipt.
- A supplied observation receipt artifact or digest differs from the report.
- Evidence counts dry-run or placeholder behavior as success.
- Agent C evidence exposes participant-private fields.
- The evidence path is unreadable or malformed.

## Acceptance Criteria

- [ ] `verify-mvp-demo` accepts optional `--agent-harness-evidence` after or
      before `--report`.
- [ ] Supplying valid Pi evidence verifies against a direct canonical report
      without adding an agent-harness claim to that report.
- [ ] Report path, actor tool receipt, canonical observation receipt, boundary,
      privacy, unreadable-file, and malformed-artifact failures remain hard
      failures with specific context.
- [ ] Verification leaves the report bytes unchanged.
- [ ] The Pi report verifier tool accepts an optional evidence path and passes
      both paths to the canonical Rust verifier.
- [ ] The evaluator runbook documents one report, one Pi evidence artifact,
      and one combined canonical verification step.
- [ ] Existing embedded agent-harness evidence and report-only verification
      remain backward compatible.

## Files To Touch

- `specs/7074-verify-pi-evidence-against-canonical-report.md`
- `crates/kamn-e2e-harness/src/mvp_demo/command_config.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_command_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/*`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_demo_runbook_contract.rs`
- `.pi/extensions/kamn-mvp/index.ts`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

- Missing flags and unknown flags return parser errors and exit non-zero.
- Unreadable evidence returns an error naming the evidence path.
- Evidence/report disagreement returns the existing specific harness boundary,
  actor receipt, observation receipt, or privacy error.
- Report-only verification keeps its current behavior.
- No failure silently falls back to report-only verification.

## Test Plan

- Red: parser contract for optional evidence and direct evidence verification.
- Red: negative direct-verification cases for report mismatch and canonical
  observation receipt mismatch.
- Red: Pi source and runbook markers for the combined verifier command.
- Green: add the optional config field, reuse existing evidence validation,
  and pass both paths through the Pi verifier tool.
- Refactor: centralize path-based evidence loading and keep modified functions
  within the repository size limits where the surrounding file permits.
- Integration: run focused command/harness/runbook tests, the broader MVP proof
  matrix, formatter, strict clippy, `make check`, local and devnet-required MVP
  demos, canonical combined verification, and a Pi agent rehearsal.

## Deviations

- The existing report-only verifier result now includes an empty
  `agent_harness_evidence` field. This is an additive JSON field; report-only
  validation behavior and exit semantics are unchanged.

## Completion Evidence

- Red command/harness/runbook contracts failed as expected because
  `--agent-harness-evidence`, the Pi parameter, and the immutable-report
  runbook flow did not exist.
- Focused green contracts passed: 23 agent-harness tests, 7 command tests, and
  1 evaluator runbook test.
- The broader MVP proof matrix passed 75 tests across claim, command, local
  artifact, three-agent transcript/view/receipt, agent-harness, and runbook
  contracts.
- `cargo fmt --check` passed.
- `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features --
  -D warnings` passed.
- `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0 make check` passed.
- Optional `make demo-mvp` returned `GO`; report-only `verify-mvp-demo`
  returned `PASS`.
- Devnet-required `make demo-mvp` returned `GO` for run
  `run-44951-1783642279986` and recorded finalized Solana signature
  `2j5TsXkw51sFs8CjhCorKC38om3YnDkmqruL7rjEsjwqSpx6pTLJVbPZBiyqqo9K8os4QhLyz3PsSgAt4bYXofQ8`.
- `solana confirm -v` independently showed the 1,000,000 lamport transfer
  finalized in slot `475171619` from
  `Ew2NpaFAK2TbUkbUMV54JN1gURSKkLWEypk5v9kJR7XU` to
  `BSN17KC1c5kUuA7ZaTXvMUnFbZUhizeaisYcAFeTsbEb`.
- Pi with `openai-codex/gpt-5.5` invoked Agent A register/invoke, Agent B
  register/accept, and Agent C verify tools, then wrote
  `/tmp/kamn-pi-7074-evidence.json` with five actor tool receipts and canonical
  observation receipts for all three agents.
- Combined verification with `--report .kamn/demo/latest/proof/report.json
  --agent-harness-evidence /tmp/kamn-pi-7074-evidence.json` returned `PASS`.
- The report SHA-256 remained
  `33590f1770c0fb5a501bd0956c26d12c354624d47f4db7962f1e2261685a775c`
  before and after combined verification.
