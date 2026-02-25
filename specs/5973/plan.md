# Plan: Issue #5973

## Approach
Deliver via three focused stories and one primary task each:
1. Cryptographic transport auth upgrade (#5974/#5977).
2. Governance/runtime assurance rebalance (#5975/#5978).
3. R57 high-gap non-regression hardening (#5976/#5979).

## Affected Modules (Expected)
- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-core/src/signature_profile.rs`
- `crates/kamn-node/src/service_api_endpoint/*`
- `.github/workflows/*` and `scripts/ci/*` (ratio/non-regression gates)

## Risks / Mitigations
- Risk: auth profile migration breaks compatibility.
  Mitigation: explicit compatibility switch, fail-closed defaults, integration tests.
- Risk: ratio gate introduces noisy CI failures.
  Mitigation: deterministic telemetry schema + bounded thresholds.
- Risk: non-regression matrix drifts from runtime behavior.
  Mitigation: map each marker to executable tests.

## Interfaces / Contracts
- Service request signature profile contract.
- Governance/runtime ratio telemetry JSON/report schema.
- R57 high-gap evidence matrix schema (tests/gates mapping).
