# 6924-split-data-layer-m2-gateway-access

## Objective

Decompose `crates/kamn-core/src/data_layer_m2_gateway_access.rs` into bounded concern-based modules while preserving M2 gateway authentication, authorization, RLS template emission, and audit-chain behavior.

## Inputs/Outputs

Inputs:
- Existing M2 gateway request/response structs, auth/session logic, ABAC policy checks, RLS emission, and audit-log verification paths in `data_layer_m2_gateway_access.rs`
- Current `kamn-core` tests and public call sites using the M2 gateway surface

Outputs:
- A thin root shell at `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- Extracted bounded modules under `crates/kamn-core/src/data_layer_m2_gateway_access/`
- A hard-fail extraction contract enforcing the root-shell/module layout
- Green real tests covering authentication, authorization, RLS, and audit behavior after the split

## Boundaries/Non-goals

In scope:
- Pure decomposition of `data_layer_m2_gateway_access.rs` by concern
- Local helper extraction needed to satisfy file/function size limits
- Import updates and contract/test updates required by the split

Out of scope:
- Redesigning M2 gateway semantics or changing public behavior
- Changing reason codes except where required for equivalent extraction
- Broad refactors outside the `data_layer_m2_gateway_access` boundary

## Failure modes

- Extraction contract does not enforce the required root-shell/module layout
- DID auth/session issuance behavior changes during extraction
- ABAC authorization or RLS template emission drifts
- Audit log hash-chain validation or export semantics change
- Touched-Rust size policy still fails because extracted files/functions remain oversized

## Acceptance criteria

- [ ] `crates/kamn-core/src/data_layer_m2_gateway_access.rs` is reduced to a thin root shell
- [ ] Concern-based modules exist under `crates/kamn-core/src/data_layer_m2_gateway_access/`
- [ ] A hard-fail extraction contract enforces the root shell and module layout
- [ ] Existing M2 gateway tests remain green on the extracted code path
- [ ] Touched-Rust size policy returns `policy_decision=GO`
- [ ] Final spec records evidence and any deviations

## Files to touch

- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/src/data_layer_m2_gateway_access/*.rs`
- `crates/kamn-core/tests/*m2*`
- `specs/6924-split-data-layer-m2-gateway-access.md`

## Error semantics

- Preserve existing typed error translation into `DataLayerM2GatewayError`
- Preserve existing reason-code mapping for auth, authorization, and audit-chain validation failures
- Do not introduce silent fallbacks, swallowed errors, or logging in interior code

## Test plan

Red:
- Add a module extraction contract that fails while `data_layer_m2_gateway_access.rs` remains monolithic and the expected module layout is absent

Green:
- Run the extraction contract
- Run the real M2 gateway target/tests that exercise auth, authorization, RLS, and audit behavior
- Run touched-Rust size policy against the issue diff

Refactor/Integration:
- Keep all extracted files under 200 LOC where possible and all touched functions under 25 LOC
- Re-run the same real tests and touched-Rust after refactor

## Proposed module seams

- `auth.rs` for DID auth/session issuance
- `authorization.rs` for ABAC visibility checks
- `rls.rs` for RLS session-setting and SQL-template emission
- `audit.rs` for append-only audit logging and hash-chain verification
- `models.rs` for shared request/response structs and enums if needed
- `tests.rs` for inline test extraction if required later

## Final evidence

- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test data_layer_m2_gateway_access_module_extraction_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core data_layer_m2_gateway_access::tests:: --lib -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6924-home-clean-1773320683 --base-ref origin/main --output-json /tmp/6924-home-touched-size.json`
- touched-Rust result: `policy_decision=GO`

## Deviations

- Current `main` required a compile-fix import in `crates/kamn-core/src/runtime_peer_coordination/tests.rs` so the targeted `kamn-core` lib test path could resolve `LIBP2P_LIVE_TRANSPORT_FEATURE_NAME` during verification. This was treated as part of the issue write set because the clean baseline otherwise failed the targeted verification path.
