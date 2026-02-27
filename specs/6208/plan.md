# Plan: Issue 6208 - Expose SDK Service Timeout Configuration

- Issue: #6208
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Replace fixed timeout constant usage with one resolver function.
2. Read `KAMN_SDK_SERVICE_TIMEOUT_SECONDS` from environment with strict validation.
3. Keep default `2s` timeout when env is absent.
4. Add unit tests for default, valid configured value, and invalid values.

## Affected Modules

- `crates/kamn-sdk/src/service.rs`

## Risks and Mitigations

1. Risk: env-based tests can race in parallel execution.
   - Mitigation: add test-local mutex guard around env mutation tests.
2. Risk: transport init behavior regression.
   - Mitigation: isolate change to timeout resolver and retain current socket timeout wiring.

