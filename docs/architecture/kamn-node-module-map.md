# KAMN Node Module Map

This map documents ownership boundaries for `crates/kamn-node` so CLI/runtime
refactors stay modular and auditable.

## Ownership Matrix

### Binary Orchestration Surface

- File:
  - `crates/kamn-node/src/main.rs`
- Ownership boundary:
  - main.rs orchestrates only: argument handoff, runtime mode dispatch,
    report assembly, and process exit mapping.
  - Do not reintroduce parser implementation into src/main.rs.
  - Do not inline signer payload, wire rendering, or live-runtime transport
    execution into `src/main.rs`.

### CLI Parsing Surface

- File:
  - `crates/kamn-node/src/cli.rs`
- Ownership boundary:
  - Owns `parse_args` and parser helper contracts for runtime-mode inputs and
    validation.
  - Produces deterministic `NodeCli` parsing outcomes consumed by orchestration.

### Kolme Live Runtime Surface

- File:
  - `crates/kamn-node/src/runtime_kolme_live.rs`
- Ownership boundary:
  - Owns live-provider execution, submit/finality handling, and deterministic
    kolme runtime status mapping.

### Signing Surface

- File:
  - `crates/kamn-node/src/signer.rs`
- Ownership boundary:
  - Owns signer-profile normalization, key-source policy enforcement, and
    secp256k1 signing adapter integration.

### Wire Payload Surface

- File:
  - `crates/kamn-node/src/wire_payload.rs`
- Ownership boundary:
  - Owns deterministic runtime-commit wire payload rendering and native message
    projection.

### Report Rendering and Assembly

- Files:
  - `crates/kamn-node/src/report_builder.rs`
  - `crates/kamn-node/src/report_render.rs`
- Ownership boundary:
  - Owns deterministic report field assembly and text/json rendering.

## Regression Marker

- `Regression: #2606`
- Guardrail intent:
  - Keep module boundaries explicit so future changes do not collapse back into
    a monolithic `main.rs` implementation surface.
