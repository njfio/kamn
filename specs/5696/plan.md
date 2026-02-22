# Plan: #5696 Activate Opt-In Live SDK-Direct S-01 Driver Execution

## Approach
1. Refactor `SdkDirectDriver` from unit struct into configurable driver with:
   - live toggle flag
   - injectable S-01 probe function for testability
2. Implement default live probe using `kamn-agent-lib::KamnAgentHandle`
   (`connect` + `health` + identity DID sanity).
3. Keep default `from_env` behavior disabled unless explicit live env flag is
   set.
4. Update run-contract driver construction to instantiate SDK driver via
   `from_env`.
5. Add unit tests for C-01..C-04 and run full harness regression suite.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/run_contract.rs`
- `crates/kamn-e2e-harness/tests/*` (only if additional integration assertion is needed)

## Risks and Mitigations
- Risk: live probe attempts network access in deterministic test runs.
  Mitigation: live mode defaults to disabled; probe not invoked unless enabled.

- Risk: behavior drift in existing contract outputs.
  Mitigation: keep run output schema unchanged and execute full harness test suite.

## Interfaces / Contracts
- `HarnessDriver` trait remains unchanged.
- `SdkDirectDriver` gains constructor surface for configurable probe behavior:
  deterministic default + opt-in live mode.

## ADR
- Not required: no dependency change, no cross-crate protocol change.
