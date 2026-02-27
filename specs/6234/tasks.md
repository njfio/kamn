# Issue 6234 Tasks

- T1 (Red): Add `kolme-live` default/override observability TLS tests and capture failing baseline.
- T2 (Green): Update TLS resolver to apply production-safe default policy for `kolme-live`.
- T3 (Green): Thread runtime mode into endpoint-server TLS resolver call.
- T4 (Green): Update runtime-network docs for default + override semantics.
- T5 (Regression): Re-run observability endpoint test slice and ensure existing TLS negative-matrix tests remain green.
- T6 (Verification): Map AC/C-cases in PR and close issue with deterministic evidence.
