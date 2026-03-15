# 6995-live-escrow-settlement-slice

## Objective
Prove one bounded live escrow settlement slice on current `main` by executing the existing `kamn-e2e-harness` S-05 escrow-settlement scenario in real external-execution `sdk-direct` mode against a local Kolme runtime and three local KAMN API nodes. Publish one operator-facing validation runbook and one hard-fail docs contract that capture the exact commands and the exact limits of the claim.

## Inputs/Outputs
### Inputs
- Current `main` binaries for:
  - `target/debug/kamn-node`
  - `target/debug/kamn-e2e-harness`
- Upstream Kolme `example-p2p` binary built locally
- Local loopback runtime endpoints:
  - Kolme API on `127.0.0.1:3000`
  - KAMN processor API on `127.0.0.1:8080`
  - KAMN listener API on `127.0.0.1:8081`
  - KAMN approver API on `127.0.0.1:8082`
- External-execution live env toggles for `sdk-direct`

### Outputs
- Validation runbook: `docs/validation/live-escrow-settlement-slice.md`
- Hard-fail docs contract: `crates/kamn-node/tests/live_escrow_settlement_slice_contract.rs`
- Runtime proof index update: `docs/validation/current-proven-runtime-slices.md`
- Review-surface wiring if needed: `docs/review/corrected-audit-response-2026-03-14.md`
- Final spec evidence in this file

## Boundaries/Non-goals
- Do not claim Solana-backed settlement.
- Do not claim bridge settlement, bridge finality, or external chain settlement.
- Do not claim Byzantine-safe or adversarial economic settlement.
- Do not add new drivers, shell lanes, or CI/workflow changes.
- Do not represent non-external-execution harness `run` output as live proof.
- Do not claim CLI-scripted or MCP-agent proof completion in this issue unless they are actually executed.

## Failure modes
- Missing or invalid Kolme binary path fails the live run.
- Missing or invalid external KAMN node binary env fails the live run.
- Missing live env toggle for `sdk-direct` causes the scenario execution to fail instead of silently passing.
- Missing or unhealthy local endpoints fail the live run.
- `verify` fails if evidence markers or chain dump are missing.
- The runbook overstates the proof as Solana-backed, bridge-backed, or external-chain settlement.
- The proof index omits the new slice.

## Acceptance criteria (testable booleans)
- [ ] `docs/validation/live-escrow-settlement-slice.md` exists.
- [ ] The runbook states that the proof is anchored to external-execution `sdk-direct` S-05 on current `main`.
- [ ] The runbook includes the exact `run` command with `--enable-external-execution` and `--scenarios S-05`.
- [ ] The runbook includes the exact `verify` command against the generated evidence directory.
- [ ] The runbook names bounded evidence markers for S-05 task/escrow settlement execution.
- [ ] The runbook states explicitly that it does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.
- [ ] `crates/kamn-node/tests/live_escrow_settlement_slice_contract.rs` fails red before the runbook exists and passes green after wiring.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the new runbook.
- [ ] The spec records exact local evidence commands and whether the executed proof was limited to `sdk-direct` only.

## Files to touch
- `specs/6995-live-escrow-settlement-slice.md`
- `docs/validation/live-escrow-settlement-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `docs/review/corrected-audit-response-2026-03-14.md`
- `crates/kamn-node/tests/live_escrow_settlement_slice_contract.rs`

## Error semantics
- The docs contract fails hard if the runbook or index wiring is missing.
- The runbook must distinguish real external-execution proof from contract-only harness execution.
- Operator commands in the runbook must fail loudly on missing binaries, env, or evidence files.
- No silent fallback from live external-execution to contract-only execution is allowed in the proof claim.

## Test plan
1. Phase 3 red test:
   - add `crates/kamn-node/tests/live_escrow_settlement_slice_contract.rs`
   - require the new runbook path, the external-execution `sdk-direct` markers, the bounded non-goal markers, and proof-index presence
   - verify the test fails before the runbook exists
2. Green/docs wiring:
   - publish `docs/validation/live-escrow-settlement-slice.md`
   - add the slice to `docs/validation/current-proven-runtime-slices.md`
   - wire the corrected audit response if needed
3. Runtime evidence:
   - build `kamn-node` and `kamn-e2e-harness`
   - build local Kolme `example-p2p`
   - start Kolme and three KAMN nodes on loopback
   - run:
     - `target/debug/kamn-e2e-harness run --mode sdk-direct --kolme-binary /tmp/kolme/target/release/example-p2p --enable-external-execution --evidence-dir /tmp/kamn-e2e-live-s05-evidence --scenarios S-05`
   - verify:
     - `target/debug/kamn-e2e-harness verify --evidence-dir /tmp/kamn-e2e-live-s05-evidence --kolme-chain-dump /tmp/kamn-e2e-live-s05-evidence/kolme_chain_dump.json --output /tmp/kamn-e2e-live-s05-verify-report.json`
4. Final checks:
   - `cargo test -p kamn-node --test live_escrow_settlement_slice_contract -- --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6995-touched-size.json`
