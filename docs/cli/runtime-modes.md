# Runtime Modes and Startup Error Contracts

This document defines startup validation behavior for `kamn-node` runtime mode
selection and preflight checks.

## Startup Control-Flow Contract

Startup and preflight validation paths must be fallible and typed. Production
startup code must not use panic-based control flow:

- disallowed in startup paths: `expect(...)`
- disallowed in startup paths: `unreachable!(...)`
- disallowed in startup paths: `panic!(...)`

Regression coverage in `cli_tests` enforces this contract over production
sections of startup-related modules.

## Deterministic Startup Errors

Startup/preflight failures return typed `ConfigError` values with deterministic
messages for operator diagnostics and contract-lane checks. Common categories:

- missing required CLI argument (`ConfigError::MissingArgumentValue`)
- runtime-mode lifecycle invariant failure (`ConfigError::RuntimeDaemonLifecycle`)
- kolm-live signer/preflight policy failure (`ConfigError::RuntimeKolmeLive`)

## Runtime Modes

- `bootstrap`: base bootstrap plan generation.
- `planning`: deterministic proposal ordering and plan shaping.
- `recovery-check`: rejoin/catch-up decision flow.
- `daemon`: tick-based runtime execution and shutdown telemetry.
- `api`: API runtime mode with endpoint controls.
- `full`: combined supervisor path with bootstrap and daemon flow contracts.
- `kolme-live`: live runtime-commit path with signer and provider preflight.
