# Tasks: Issue #5961 - Close escaped http_transport mutants from #5932 mutation gate

- Issue: #5961
- Spec: `specs/5961/spec.md`
- Plan: `specs/5961/plan.md`
- Status: Draft
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED): add tests targeting 8 escaped mutant points in `http_transport.rs`.
- T2 (GREEN): run `cargo test -p kamn-core` targeted to new tests and fix only if contract mismatch appears.
- T3 (VERIFY): run mutation against `http_transport.rs` scope and capture totals.
- T4 (PROCESS): post evidence to #5957 and #5961; update status/docs.
