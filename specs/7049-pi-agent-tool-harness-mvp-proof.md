# 7049 - Pi Agent Tool Harness MVP Proof

## Objective

Add a project-local Pi extension that gives Pi named KAMN MVP proof tools for the
existing demo/report/verifier path. The harness should let an evaluator run Pi
with `openai-codex/gpt-5.5`, invoke KAMN-specific tools, create the optional
agent-harness evidence artifact, run `make demo-mvp`, and verify the final proof
report without relying on an ad hoc bash-only prompt.

## Inputs/Outputs

- Input: existing `.kamn/demo/latest/proof/report.json`.
- Input: optional Pi tool output path for the agent-harness evidence artifact.
- Input: existing local `make demo-mvp` target and `verify-mvp-demo` command.
- Output: a Pi-generated
  `kamn.mvp.agent-harness-evidence.v1` artifact.
- Output: `.kamn/demo/latest/proof/report.json` with
  `mcp_agent_harness_verification` when the evidence artifact is supplied.
- Output: `.kamn/demo/latest/proof/report.md` that surfaces the optional agent
  harness evidence artifact and claim boundary when present.

## Boundaries/Non-goals

- The Pi extension proves Pi custom tools can drive the KAMN proof path. It does
  not claim generic Pi MCP protocol support unless a Pi MCP bridge exists.
- The extension must not read `.kamn/devnet/*.env`, private keys, OAuth tokens,
  or any committed secret material.
- Settlement or asset movement remains devnet-backed only when the JSON report
  carries devnet evidence. Local Pi tool execution must not promote local-only or
  dry-run settlement to MVP success.
- No production readiness, mainnet behavior, or real economic value movement is
  claimed.
- No new npm/Rust dependencies are required.

## Failure Modes

- Pi extension fails to load: the Pi smoke must fail before claiming harness
  evidence.
- Pi tool writes malformed evidence: `verify-mvp-demo` must reject the report.
- Evidence claims private verifier visibility: `verify-mvp-demo` must reject the
  report.
- Evidence claims local-only settlement: `verify-mvp-demo` must reject the
  report.
- Report Markdown omits present agent-harness evidence: Markdown contract tests
  must fail.
- Pi smoke cannot authenticate or use the configured `openai-codex/gpt-5.5`
  model: record the blocker and keep the core Rust proof path green.

## Acceptance Criteria

- [x] A project-local Pi extension registers named KAMN MVP proof tools for
  report verification, claim-boundary inspection, evidence artifact writing, and
  local demo execution with an evidence artifact.
- [x] The Pi-generated artifact uses schema
  `kamn.mvp.agent-harness-evidence.v1`, participant markers for
  `agent_a`, `agent_b`, `agent_c_verifier`, and tool markers for `register`,
  `create_task`, `fund_escrow`, `release_escrow`, and `verify_proof`.
- [x] The artifact records `execution_surface:"pi-extension-tools"` or an
  equally explicit Pi-tool marker, while the final report remains honest about
  what is local-only and what is devnet-backed.
- [x] `verify-mvp-demo` rejects missing, malformed, private-leaking, local-only
  settlement, or placeholder/dry-run-counting harness evidence.
- [x] The human-readable report includes the optional agent-harness artifact and
  claim when present.
- [x] A Pi non-interactive smoke using `openai-codex/gpt-5.5` loads the extension
  and leaves a report that passes
  `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`.
- [x] Documentation states this proves the Pi extension/tool path, not generic Pi
  MCP protocol support.

## Files to Touch

- `.pi/extensions/kamn-mvp/index.ts`
- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report_markdown.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/support.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

- Pi custom tools must throw on invalid report paths, malformed report JSON,
  failed verifier commands, failed demo commands, or unsafe secret-like paths.
- Rust verifier code must return explicit error strings naming the violated
  harness evidence field.
- Entrypoint output must not silently downgrade failed harness evidence into a
  passing local-only result.

## Test Plan

1. Red: add a contract test that a report generated with agent-harness evidence
   includes that evidence in `report.md`.
2. Red: add a verifier contract that accepts `pi-extension-tools` explicitly
   while preserving the existing `mcp-tools` evidence surface from #7047.
3. Red: add a file-existence/marker contract for
   `.pi/extensions/kamn-mvp/index.ts` requiring the named KAMN tools.
4. Green: implement the minimal Pi extension and Rust verifier/report changes.
5. Refactor: keep functions small, no new dependencies, and preserve existing
   claim semantics.
6. Integration: run a local Pi smoke with the extension and no repository file
   edits, then verify the final report with `verify-mvp-demo`.
7. Full verification: `cargo fmt --check`, strict clippy, `make check`, targeted
   MVP demo tests, `make demo-mvp`, Pi smoke, and final report verifier.

## Completion Evidence

- Spec committed and linked to issue #7049 before implementation.
- Red test output captured:
  `mvp_demo_agent_harness_claim_contract` failed on unsupported
  `pi-extension-tools`, missing Markdown harness evidence, and missing
  `.pi/extensions/kamn-mvp/index.ts`.
- Red docs output captured:
  `mvp_evaluator_demo_runbook_contract` failed on the missing
  `## Optional Pi Agent Harness` section.
- Targeted tests green after implementation:
  `mvp_demo_agent_harness_claim_contract`,
  `mvp_evaluator_demo_runbook_contract`, `mvp_demo_command_contract`,
  `mvp_demo_claim_contract`, and `mvp_demo_three_agent_claim_contract`.
- `cargo fmt --check` green.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` green.
- `make check` green.
- `make demo-mvp` green with local-only `GO` report.
- Pi non-interactive smoke with `openai-codex/gpt-5.5` and only the project-local
  KAMN extension tools enabled passed all five steps.
- Final verifier passed:
  `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json`.
- PR links issue #7049 and includes test evidence.
