# Tasks: Issue #5909 - Fail Closed Insecure Deterministic Message Crypto by Default

1. T1 (RED): add failing tests for direct/group constructor behavior when opt-in env markers are absent.
2. T2 (GREEN): remove debug auto-enable and require explicit env opt-in in both crypto modules.
3. T3 (REFACTOR): ensure env parsing and constructor errors remain deterministic.
4. T4 (VERIFY): run fmt, clippy, and targeted `kamn-core` tests for changed modules.
5. T5 (REGRESSION): verify debug/test profile cannot implicitly re-enable insecure deterministic crypto.
