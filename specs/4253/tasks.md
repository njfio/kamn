# Tasks — #4253 Finality Evidence Convergence Checker

Status: Reviewed

- T1 (Conformance/Regression): add red tests for missing-link, payload tamper, and promotion-reason mapping drift (`#4259`).
- T2 (Implementation): add deterministic promotion reason mapping and evidence-convergence subcommand (`#4260`).
- T3 (Integration): wire convergence checker into libp2p contract lane + lane report markers.
- T4 (Docs): update planning/runbook/release checklist marker contracts and command references.
- T5 (Verification): run targeted script + doc-contract tests, then clippy/fmt gates.
