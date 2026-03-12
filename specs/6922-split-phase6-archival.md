# 6922-split-phase6-archival

## Objective

Decompose `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs` into bounded concern-based modules while preserving phase-6 partition archival behavior, policy validation, and runtime-evidence projection semantics.

## Inputs/Outputs

Inputs:
- Existing phase-6 scheduler/runtime policy inputs, runtime evidence structures, and partition archival requests already handled by `phase6.rs`
- Current `kamn-core` and `kamn-data-layer` integration points used by the phase-6 archival path

Outputs:
- A thin root shell at `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- Extracted bounded modules under `crates/kamn-core/src/data_layer_m10_partition_archival/phase6/`
- A hard-fail extraction contract that enforces the new layout
- Green real tests covering phase-6 archival behavior after the split

## Boundaries/Non-goals

In scope:
- Pure decomposition of `phase6.rs` by concern
- Local helper extraction required to satisfy file/function size limits
- Updating internal imports and tests required by the split

Out of scope:
- Redesigning partition archival semantics
- Changing external APIs or reason-code behavior except where required for equivalent extraction
- Broad refactors outside the phase-6 archival boundary

## Failure modes

- Extraction contract does not detect the required root-shell/module layout
- Phase-6 scheduler/runtime policy translation drifts during extraction
- Runtime evidence projection or owner-scope validation changes behavior
- Port/adapter error mapping changes reason codes or typed error translation
- Touched-Rust size policy still fails because extracted files/functions remain oversized

## Acceptance criteria

- [ ] `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs` is reduced to a thin root shell
- [ ] Concern-based modules exist under `crates/kamn-core/src/data_layer_m10_partition_archival/phase6/`
- [ ] A hard-fail extraction contract enforces the root shell and module layout
- [ ] Existing phase-6 archival tests remain green on the extracted code path
- [ ] Touched-Rust size policy returns `policy_decision=GO`
- [ ] Final spec records evidence and any deviations

## Files to touch

- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6/*.rs`
- `crates/kamn-core/tests/*phase6*`
- `specs/6922-split-phase6-archival.md`

## Error semantics

- Preserve existing typed error translation into `DataLayerM10PartitionLifecycleError`
- Preserve existing reason-code mapping for phase-6 scheduler/runtime validation and execution failures
- Do not introduce silent fallbacks, swallowed errors, or logging in interior code

## Test plan

Red:
- Add a module extraction contract that fails while `phase6.rs` remains monolithic and the expected module layout is absent

Green:
- Run the extraction contract
- Run the real phase-6 archival target/tests that exercise scheduler/runtime policy and evidence projection behavior
- Run touched-Rust size policy against the issue diff

Refactor/Integration:
- Keep all extracted files under 200 LOC where possible and all touched functions under 25 LOC
- Re-run the same real tests and touched-Rust after refactor

## Proposed module seams

- `adapters.rs` for M8/phase-6 projection bridge types and error translation
- `runtime_evidence.rs` for runtime-evidence bundle/report/state mapping
- `policy_mapping.rs` for core/policy budget and scheduler translation helpers
- `scheduler.rs` for cycle report, overflow projection, validation, and preflight evaluation helpers
- `models.rs` for scheduler/runtime constructors and associated impl blocks if needed
- `tests.rs` for any inline test extraction if present or required later

## Phase 6 Evidence

- `cargo test -p kamn-core --test data_layer_m10_phase6_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6922-touched-size.json`
- Result: extraction contract `PASS`, real phase-6 archival target `PASS (38 passed)`, touched-Rust `policy_decision=GO`

## Deviations

- The dedicated integration target for this issue is `--test data_layer_m10_partition_archival`, not `cargo test -p kamn-core phase6 --lib`, because the broader library path currently hits an unrelated compile problem in `runtime_peer_coordination/tests.rs`.
- Clean-clone touched-Rust validation was not required for this issue because the primary checkout was already back on a clean `main`; the direct Python entrypoint was used for deterministic repo-root resolution.
