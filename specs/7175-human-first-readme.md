# 7175 Human-First README

## Objective

Make the repository README understandable to a new human reader in its first
screen, then progressively expose KAMN's runtime architecture, receipt-authority
flow, proof boundaries, repository map, and contributor guidance for agents and
maintainers.

## Inputs / Outputs

Inputs:
- the current workspace crate graph
- the architecture navigation index and service delivery flow
- the current proven runtime-slices index
- the existing 200-line README compact contract
- the README contract reference for deep command and policy detail

Outputs:
- a README of no more than 200 lines
- one inline Mermaid component architecture diagram
- one inline Mermaid receipt-authority sequence diagram
- a current, linked workspace map
- contract tests for progressive section order and diagram markers

## Boundaries / Non-Goals

- Do not change runtime behavior, public APIs, dependencies, CI, or tooling.
- Do not rewrite detailed architecture or validation documents.
- Do not claim production readiness, mainnet custody, generalized settlement,
  or live economic movement without current proof.
- Do not move the command-marker inventory back into the compact README.
- Do not exceed 200 lines to make the README exhaustive.

## Failure Modes

- Product language implies capabilities beyond current proof.
- Quickstart requires a funded external run before offering a local path.
- Architecture omits SDK, CLI, MCP, service runtime, proof, or authority edges.
- The flow treats actor identity as settlement authority without receipt
  digests and finalized settlement evidence.
- Mermaid diagrams are absent or their stable markers drift.
- Sections regress from progressive disclosure to reference-first ordering.
- The crate map names missing crates or omits active workspace groups.
- A local documentation link resolves to no repository file.
- Existing README contract markers or the 200-line cap regress.

## Acceptance Criteria

- [x] The opening states what KAMN is, who it serves, and its maturity boundary.
- [x] A local quickstart precedes evaluator and implementation detail.
- [x] Architecture and authority-flow Mermaid diagrams are present and current.
- [x] Receipt digests, authorization, finality, and receipts are explicit.
- [x] Sections progress from human overview to agent/maintainer depth.
- [x] The workspace map covers all active crate responsibility groups.
- [x] The README is no more than 200 lines and all local links resolve.
- [x] New progressive-disclosure contracts fail before the README rewrite.
- [x] Existing and new README contract tests pass after the rewrite.
- [x] Formatting and applicable Clippy checks pass.

## Files To Touch

- `specs/7175-human-first-readme.md`
- `README.md`
- `crates/kamn-core/tests/readme_compact_contract.rs`
- `crates/kamn-core/tests/readme_contract_lane.rs`

## Error Semantics

Documentation contract failures must identify the missing heading, diagram
marker, ordering boundary, line-count breach, or unresolved local link. Tests
must fail closed; no optional fallback accepts an incomplete README.

## Test Plan

Red:
- Require the new progressive section headings in strict order.
- Require stable Mermaid component and sequence diagram markers.
- Require explicit service-receipt authority language.
- Require every relative Markdown link in the README to resolve.

Green:
- Rewrite the README with concise human-first copy and a local quickstart.
- Add accurate component and receipt-authority flow diagrams.
- Group all active workspace crates by responsibility and link deeper maps.

Refactor:
- Remove repeated evaluator and command detail already owned by linked docs.
- Keep sections skimmable, claims evidence-bound, and the file below 200 lines.
- Verify headings and diagram labels communicate without surrounding prose.

Integration:
- Run the new progressive-disclosure contract.
- Run `readme_compact_contract` and `readme_contract_lane`.
- Run formatting and Clippy for the affected test target.

## Verification Evidence

- RED: `cargo test -p kamn-core --test readme_compact_contract --test
  readme_contract_lane` failed on the missing `## Why KAMN` heading and
  `diagram:kamn-runtime-architecture` marker before the README rewrite.
- GREEN: the same command passed six compact and three lane tests after the
  rewrite.
- Integration: `bash scripts/ci/test_readme_contract.sh` passed through the
  repository's isolated README target.
- Workspace quality: `make check` passed `cargo fmt --check` and
  workspace-wide, all-target, all-feature Clippy with warnings denied.
- Compactness: `README.md` is 181 lines; the link contract resolved every local
  Markdown target.

## Refactor Checklist

- [x] Every touched Rust function is 25 lines or fewer.
- [x] Every touched file is 200 lines or fewer.
- [x] Repository-root construction is centralized in the compact contract.
- [x] README command inventory remains in the contract reference.
- [x] No dependencies, runtime behavior, CI, shell surfaces, or APIs changed.
- [x] No TODOs, placeholders, dead tests, or silent fallbacks were introduced.
