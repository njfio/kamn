# Plan: #5778 Reconcile R53 Portable-Agent Stalled Markers After Query-Surface Delivery

## Approach
1. Add deterministic post-publication marker block to `docs/review/gaps-and-issues-r53.md`:
   - schema/version marker
   - baseline stall status marker
   - post-publication status marker
   - evidence links (`#5776`, `#5777`)
   - delta markers for CLI/MCP surface counts.
2. Keep existing snapshot table/text intact to preserve "As of" semantics.
3. Extend `review_r53_docs_contract.rs` to assert:
   - required new marker keys exist,
   - marker values are internally consistent,
   - evidence issue/PR markers are non-empty and correctly formatted.
4. Run RED->GREEN contract lane and quality gates.
5. Preserve spec cap via one compensating archive cleanup entry removal.

## Affected Modules
- `docs/review/gaps-and-issues-r53.md`
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `specs/archive/index.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: Breaking existing marker contracts.
  - Mitigation: Keep existing required keys unchanged; add new assertions incrementally.
- Risk: Exceeding top-level specs cap.
  - Mitigation: Remove one archived pointer/spec pair and update archive index counts.

## Interfaces / Contracts
- Review marker lines use `- key=value` schema and are parsed by `review_r53_docs_contract`.
- New markers must remain deterministic and independent of volatile runtime state.

## ADR
- Not required (no architecture/protocol/dependency decision).
