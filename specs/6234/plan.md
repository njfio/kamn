# Issue 6234 Plan

## Approach
1. Add test coverage first in `observability_endpoint_tests` for:
   - `kolme-live` default fail-closed behavior (missing TLS env fails),
   - explicit disabled override behavior (HTTP serve succeeds).
2. Update TLS mode resolver to accept runtime mode and apply default policy:
   - `kolme-live` + mode env absent => `Require` from cert/key env,
   - other modes + mode env absent => `Disabled`.
3. Preserve existing explicit env mode parsing (`disabled|require`) and error taxonomy.
4. Update runtime network docs to match the new default/override policy.
5. Run targeted tests and format checks.

## Affected Modules
- `crates/kamn-node/src/observability_endpoint/tls_mode.rs`
- `crates/kamn-node/src/observability_endpoint/endpoint_server.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs`
- `docs/foundation/runtime-network.md`
- `specs/6234/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: broad default change could break local daemon tests/workflows.
  - Mitigation: scope fail-closed default to `kolme-live` runtime mode; keep explicit `disabled` override.
- Risk: nondeterministic environment leakage across tests.
  - Mitigation: use existing env lock + `EnvVarGuard` patterns in test cases.
- Risk: contract drift between docs and runtime behavior.
  - Mitigation: update docs in same PR and run existing observability doc contract tests.

## Interfaces
- Internal resolver signature change: runtime mode parameter added.
- No external API/wire-format changes.
