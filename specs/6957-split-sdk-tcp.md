# 6957 Split SDK TCP

## Objective
Split `crates/kamn-sdk/src/tcp.rs` into bounded concern-based modules so the root becomes a thin shell under the active size policy while preserving current TCP SDK behavior and public API shape.

## Inputs/Outputs
- Input: existing TCP SDK implementation in `crates/kamn-sdk/src/tcp.rs`
- Output: thin root shell plus extracted modules for envelope parsing/verification, handshake flow, transport helpers, shared support, and tests

## Boundaries/Non-goals
- Do not change TCP protocol semantics
- Do not change public SDK API behavior beyond structural refactoring
- Do not add new transports or new dependencies
- Do not weaken existing TCP regression coverage

## Failure modes
- Extraction contract does not fail when the root regresses into a monolith
- Extracted modules exceed file-size policy
- Public API re-exports drift and downstream callers fail to compile
- Handshake or envelope verification behavior changes during extraction
- Replay/reconnect or tamper rejection tests regress after module movement

## Acceptance criteria
- [ ] `crates/kamn-sdk/src/tcp.rs` is reduced to a thin root shell under the active size policy
- [ ] concern-based modules exist for envelope parsing/verification, wire framing/transport flow, handshake/session flow, shared support, and tests
- [ ] existing TCP SDK behavior remains green through real crate tests
- [ ] a hard-fail extraction contract prevents the root file from regressing into a monolith
- [ ] touched-Rust size policy returns `GO` for the final write set

## Files to touch
- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-sdk/src/tcp/`
- `crates/kamn-sdk/tests/tcp_module_extraction_contract.rs`
- `specs/6957-split-sdk-tcp.md`

## Error semantics
- Preserve current `SdkError` behavior exactly
- Keep hard-fail parsing and validation behavior for malformed wire payloads, handshake mismatches, replay detection, and cryptographic verification failures
- Do not add silent fallbacks or behavior changes at module boundaries

## Test plan
- Red extraction contract for root shell budget, extracted module presence, and root markers
- Real TCP SDK tests covering envelope validation, handshake constant-time behavior, replay/reconnect, tamper rejection, and relay flow
- Touched-Rust size policy on the final write set
