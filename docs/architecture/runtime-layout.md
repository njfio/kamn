# Runtime Layout

## `kamn-node` runtime/test module layout

`crates/kamn-node/src/main_tests.rs` is organized into domain-scoped modules:

- `cli_contract_tests` for startup/argument contract validation.
- `signer_tests` for signer adapter/key-source policy behavior.
- `runtime_tests` for core runtime and Kolme live execution behavior.
- `daemon_tests` for daemon-mode lifecycle and shutdown controls.
- `report_tests` for bootstrap/report rendering and deterministic output checks.
- `core_behavior_tests` compatibility wrappers for legacy test selectors still referenced by automation contracts.

Supporting runtime-focused modules remain scoped by responsibility:

- `cli.rs` for CLI/config parsing and mode resolution.
- `runtime_kolme_live.rs` for Kolme live runtime orchestration.
- `daemon_shutdown.rs` and `daemon_observability.rs` for daemon lifecycle/telemetry.
- `report_builder.rs` and `report_render.rs` for runtime report shaping/rendering.
- `service_api_endpoint.rs` and `observability_endpoint.rs` for local API/observability surfaces.

This split keeps test ownership aligned with runtime domains while preserving backward-compatible selector coverage for existing contract lanes.
