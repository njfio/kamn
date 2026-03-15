# 6995-live-escrow-settlement-slice

## Objective
Prove one bounded live escrow settlement slice on current `main` by executing the existing `kamn-e2e-harness` S-05 escrow-settlement probe through a real local-heavy `sdk-direct` integration test against a local Kolme runtime and three local KAMN API nodes. Publish one operator-facing validation runbook and one hard-fail docs contract that capture the exact commands and the exact limits of the claim.

## Inputs/Outputs
### Inputs
- Current `main` binaries for:
  - `target/debug/kamn-node`
  - `target/debug/kamn-e2e-harness`
- Upstream Kolme `example-p2p` binary built locally
- Local loopback runtime endpoints:
  - Kolme API on `127.0.0.1:13000`
  - KAMN processor API on `127.0.0.1:18080`
  - KAMN listener API on `127.0.0.1:18081`
  - KAMN approver API on `127.0.0.1:18082`
- External-execution live env toggles for `sdk-direct`

### Outputs
- Validation runbook: `docs/validation/live-escrow-settlement-slice.md`
- Hard-fail docs contract: `crates/kamn-node/tests/live_escrow_settlement_slice_contract.rs`
- Explicit local-heavy integration proof: `crates/kamn-e2e-harness/tests/live_s05_sdk_direct_external_execution.rs`
- Runtime proof index update: `docs/validation/current-proven-runtime-slices.md`
- Review-surface wiring if needed: `docs/review/corrected-audit-response-2026-03-14.md`
- Final spec evidence in this file

## Boundaries/Non-goals
- Do not claim Solana-backed settlement.
- Do not claim bridge settlement, bridge finality, or external chain settlement.
- Do not claim Byzantine-safe or adversarial economic settlement.
- Do not add new drivers, shell lanes, or CI/workflow changes.
- Do not represent non-external-execution harness `run` output as live proof.
- Do not represent the harness external-execution orchestration shell alone as the live settlement proof; the explicit local-heavy integration test is the proof anchor.
- Do not claim CLI-scripted or MCP-agent proof completion in this issue unless they are actually executed.

## Failure modes
- Missing or invalid Kolme binary path fails the live run.
- Missing or invalid external KAMN node binary env fails the live run.
- Missing live env toggle for `sdk-direct` causes the scenario execution to fail instead of silently passing.
- Missing or unhealthy local endpoints fail the live run.
- The explicit local-heavy integration test fails against the running local runtime.
- `verify` fails if evidence markers or chain dump are missing.
- The runbook overstates the proof as Solana-backed, bridge-backed, or external-chain settlement.
- The proof index omits the new slice.

## Acceptance criteria (testable booleans)
- [ ] `docs/validation/live-escrow-settlement-slice.md` exists.
- [ ] The runbook states that the proof is anchored to external-execution `sdk-direct` S-05 on current `main`.
- [ ] The runbook names the explicit local-heavy integration test command as the proof anchor.
- [ ] The runbook includes the exact `run` command with `--enable-external-execution` and `--scenarios S-05`.
- [ ] The runbook includes the exact `verify` command against the generated evidence directory.
- [ ] The runbook names bounded evidence markers for S-05 task/escrow settlement execution.
- [ ] The runbook states explicitly that it does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.
- [ ] `crates/kamn-node/tests/live_escrow_settlement_slice_contract.rs` fails red before the runbook exists and passes green after wiring.
- [ ] `crates/kamn-e2e-harness/tests/live_s05_sdk_direct_external_execution.rs` executes `S-05` against the running local runtime when invoked explicitly with `--ignored`.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the new runbook.
- [ ] The spec records exact local evidence commands and whether the executed proof was limited to `sdk-direct` only.

## Files to touch
- `specs/6995-live-escrow-settlement-slice.md`
- `docs/validation/live-escrow-settlement-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `docs/review/corrected-audit-response-2026-03-14.md`
- `crates/kamn-node/tests/live_escrow_settlement_slice_contract.rs`
- `crates/kamn-e2e-harness/tests/live_s05_sdk_direct_external_execution.rs`

## Error semantics
- The docs contract fails hard if the runbook or index wiring is missing.
- The runbook must distinguish real external-execution proof from contract-only harness execution.
- The explicit local-heavy integration test must fail loud on missing live env or unavailable local runtime.
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
   - run the explicit local-heavy proof:
     - `cargo test -p kamn-e2e-harness --test live_s05_sdk_direct_external_execution integration_live_s05_sdk_direct_escrow_settlement_probe_against_local_runtime -- --ignored --exact --nocapture`
   - record the surrounding external-execution contract output:
     - `target/debug/kamn-e2e-harness run --mode sdk-direct --kolme-binary /tmp/kolme/target/release/example-p2p --enable-external-execution --evidence-dir /tmp/kamn-e2e-live-s05-evidence --scenarios S-05`
   - verify the generated evidence contract output:
     - `target/debug/kamn-e2e-harness verify --evidence-dir /tmp/kamn-e2e-live-s05-evidence --kolme-chain-dump /tmp/kamn-e2e-live-s05-evidence/kolme_chain_dump.json --output /tmp/kamn-e2e-live-s05-verify-report.json`
4. Final checks:
   - `cargo test -p kamn-node --test live_escrow_settlement_slice_contract -- --nocapture`
   - `cargo test -p kamn-e2e-harness --test live_s05_sdk_direct_external_execution integration_live_s05_sdk_direct_escrow_settlement_probe_against_local_runtime -- --ignored --exact --nocapture`
   - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6995-touched-size.json`

## Deviations / Final evidence
- The initial plan treated the external-execution harness `run` surface as the likely proof anchor. The actual runtime work showed that `run` still records scaffolded orchestration/evidence sections even when scenario execution is real.
- To keep the claim honest, the explicit proof anchor became `crates/kamn-e2e-harness/tests/live_s05_sdk_direct_external_execution.rs`, which calls the real `sdk-direct` S-05 probe against the running local runtime.
- The proof executed only `sdk-direct`; CLI-scripted and MCP-agent parity remain out of scope for this issue.
- Exact executed commands:
  - `cargo build -q -p kamn-node -p kamn-e2e-harness`
  - `git clone https://github.com/fpco/kolme /tmp/kolme` (if absent)
  - `cd /tmp/kolme && RUSTFLAGS='-C link-arg=-fuse-ld=bfd' cargo build --release -p example-p2p`
  - `/tmp/kolme/target/release/example-p2p api-server --bind 127.0.0.1:13000`
  - `KAMN_SERVICE_API_TLS_MODE=disabled target/debug/kamn-node --runtime-mode api --role processor --api-bind 127.0.0.1:18080 --api-max-requests 1000 --api-idle-timeout-ms 600000 --storage-dir /tmp/kamn-node-live-processor-6995`
  - `KAMN_SERVICE_API_TLS_MODE=disabled target/debug/kamn-node --runtime-mode api --role listener --api-bind 127.0.0.1:18081 --api-max-requests 1000 --api-idle-timeout-ms 600000 --storage-dir /tmp/kamn-node-live-listener-6995`
  - `KAMN_SERVICE_API_TLS_MODE=disabled target/debug/kamn-node --runtime-mode api --role approver --api-bind 127.0.0.1:18082 --api-max-requests 1000 --api-idle-timeout-ms 600000 --storage-dir /tmp/kamn-node-live-approver-6995`
  - `KAMN_E2E_SDK_DIRECT_LIVE=true KAMN_ENDPOINT=http://127.0.0.1:18080 KAMN_KOLME_ENDPOINT=http://127.0.0.1:13000 KAMN_AGENT_NAME=kamn-live-s05-proof cargo test -p kamn-e2e-harness --test live_s05_sdk_direct_external_execution integration_live_s05_sdk_direct_escrow_settlement_probe_against_local_runtime -- --ignored --exact --nocapture`
  - `KAMN_E2E_SDK_DIRECT_LIVE=true KAMN_ENDPOINT=http://127.0.0.1:18080 KAMN_KOLME_ENDPOINT=http://127.0.0.1:13000 KAMN_E2E_EXTERNAL_KAMN_PROCESSOR_BINARY=/home/n/Code/kamn/target/debug/kamn-node KAMN_E2E_EXTERNAL_KAMN_LISTENER_BINARY=/home/n/Code/kamn/target/debug/kamn-node KAMN_E2E_EXTERNAL_KAMN_APPROVER_BINARY=/home/n/Code/kamn/target/debug/kamn-node target/debug/kamn-e2e-harness run --mode sdk-direct --kolme-binary /tmp/kolme/target/release/example-p2p --enable-external-execution --evidence-dir /tmp/kamn-e2e-live-s05-evidence --scenarios S-05 > /tmp/kamn-e2e-live-s05-run.json`
  - `target/debug/kamn-e2e-harness verify --evidence-dir /tmp/kamn-e2e-live-s05-evidence --kolme-chain-dump /tmp/kamn-e2e-live-s05-evidence/kolme_chain_dump.json --output /tmp/kamn-e2e-live-s05-verify-report.json > /tmp/kamn-e2e-live-s05-verify.json`
- Results:
  - explicit live `sdk-direct` S-05 integration test: `PASS`
  - harness external-execution `run` command: `PASS` with `scenario_results=[{\"id\":\"S-05\",\"status\":\"PASS\"}]`
  - harness `verify` command: `schema_check=PASS`, `proof_check=PASS`, `chain_check=PASS`, `content_check=PASS`
  - touched-Rust: `policy_decision=GO`
