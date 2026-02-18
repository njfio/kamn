# Issue #3630 Plan

- Issue: `#3630`
- Status: `Completed`

## Approach
- Finish TLS coverage on observability path and preserve service-path parity.
- Wire TLS evidence into go/no-go contract surfaces.
- Keep deep TLS checks local-heavy and protect CI-fast budgets.

## Affected Modules
- `scripts/runtime/`
- `scripts/deploy/`
- `scripts/ci/`
- `docs/foundation/runtime-network.md`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: partial TLS coverage between endpoint families.
- Mitigation: split policy and contract-lane checks for both service and observability.
- Risk: expensive validation in default CI gate.
- Mitigation: explicit CI exclusion policy checks for local-heavy lanes.

## Interface Contract
- Endpoint TLS semantics remain deterministic with fail-closed markers.
- Go/no-go evidence schema includes required TLS fields.

## ADR
- No ADR required (governance/test hardening on existing TLS interfaces).
