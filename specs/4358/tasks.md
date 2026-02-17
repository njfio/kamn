# Tasks: #4358 Rotation Preflight Taxonomy Contract

- T1 (Conformance/Regression): add failing tests for missing rotation preflight taxonomy outputs.
- T2 (Implementation): add taxonomy constants and deterministic observed-value mapping output fields.
- T3 (Docs): update key-management rotation preflight evidence matrix markers.
- T4 (Verification): run targeted checker/contract tests + fmt/clippy/test gates.

## Tier Mapping

- Unit: checker branch coverage via fixture mutations.
- Functional: deployment preflight checker CLI pass/fail behavior.
- Conformance: C-01..C-04.
- Integration: preflight lane rotate-ready and rotate-blocked scenarios.
- Regression: stale rotation rehearsal and production key-source mismatch fail-closed proofs.
