# Tasks: #4357 Multi-Signer Quorum + Signature-Decision Taxonomy

- T1 (Conformance/Regression): add failing tests for missing signature-decision taxonomy markers and quorum drift reason mapping.
- T2 (Implementation): add signature-decision taxonomy constants and mapping outputs.
- T3 (Docs): add ops configuration markers for multi-signer profile/quorum evidence.
- T4 (Verification): run targeted scripts + fmt/clippy/test gates.

## Tier Mapping

- Unit: checker branch checks via deterministic fixture mutations.
- Functional: CLI policy pass/fail behavior.
- Conformance: C-01..C-04 checks.
- Integration: runner-generated primary/secondary profile outputs.
- Regression: quorum drift and shortfall fail-closed proofs.
