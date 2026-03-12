# 6926-split-bootstrap

## Objective

Decompose `crates/kamn-core/src/bootstrap.rs` into bounded concern-based modules while preserving bootstrap planning, transport-profile selection, runtime persistence layout resolution, and validation behavior.

## Inputs/Outputs

Inputs:
- Existing bootstrap entrypoints and helpers in `crates/kamn-core/src/bootstrap.rs`
- Current `kamn-core` tests and production call sites using bootstrap planning

Outputs:
- A thin root shell at `crates/kamn-core/src/bootstrap.rs`
- Extracted bounded modules under `crates/kamn-core/src/bootstrap/`
- A hard-fail extraction contract enforcing the root-shell/module layout
- Green real tests covering bootstrap behavior after the split

## Boundaries/Non-goals

In scope:
- Pure decomposition of `bootstrap.rs` by concern
- Local helper extraction needed to satisfy file/function size limits
- Import updates and contract/test updates required by the split

Out of scope:
- Changing bootstrap semantics or config behavior
- Broad runtime redesign outside the `bootstrap` boundary
- Weakening existing tests or policy gates

## Failure modes

- Extraction contract does not enforce the required root-shell/module layout
- Transport-profile selection behavior changes during extraction
- Runtime persistence layout ordering or validation drifts
- Config error mapping changes during helper extraction
- Touched-Rust size policy still fails because extracted files/functions remain oversized

## Acceptance criteria

- [ ] `crates/kamn-core/src/bootstrap.rs` is reduced to a thin root shell
- [ ] Concern-based modules exist under `crates/kamn-core/src/bootstrap/`
- [ ] A hard-fail extraction contract enforces the root shell and module layout
- [ ] Existing bootstrap tests remain green on the extracted code path
- [ ] Touched-Rust size policy returns `policy_decision=GO`
- [ ] Final spec records evidence and any deviations

## Files to touch

- `crates/kamn-core/src/bootstrap.rs`
- `crates/kamn-core/src/bootstrap/*.rs`
- `crates/kamn-core/tests/*bootstrap*`
- `specs/6926-split-bootstrap.md`

## Error semantics

- Preserve existing typed `ConfigError` behavior and wrapped validation causes
- Preserve existing bootstrap validation and path-check failure semantics
- Do not introduce silent fallbacks, swallowed errors, or logging in interior code

## Test plan

Red:
- Add a module extraction contract that fails while `bootstrap.rs` remains monolithic and the expected module layout is absent

Green:
- Run the extraction contract
- Run the real bootstrap target/tests that exercise bootstrap planning and persistence-layout validation
- Run touched-Rust size policy against the issue diff

Refactor/Integration:
- Keep all extracted files under 200 LOC where possible and all touched functions under 25 LOC
- Re-run the same real tests and touched-Rust after refactor

## Proposed module seams

- `entrypoints.rs` for public bootstrap entrypoints
- `layout.rs` for runtime persistence layout resolution and prioritization
- `validation.rs` for store-path validation helpers
- `error_mapping.rs` for storage-to-config error translation
- `tests.rs` for inline test extraction if required

## Final evidence

- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test bootstrap_module_extraction_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core bootstrap::tests:: --lib -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-clean-20260312-091752 --base-ref origin/main --output-json /tmp/6926-touched-size-staged.json`
- touched-Rust result: `policy_decision=GO`

## Deviations

- None.
