# Issue #5199 Tasks

- Issue: #5199
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Ordered Tasks
- T1 (Tests/RED): add a regression that poisons signer env lock and fails while direct `.expect(...)` lock callsites remain.
- T2 (Implementation/GREEN): replace remaining `signer_env_lock().lock().expect(...)` callsites with `lock_signer_env_guard()`.
- T3 (Verification): run targeted `kamn-node` signer and CLI suites and confirm stable pass.
- T4 (Process): update issue process log, status, and PR AC/test-tier evidence.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | lock helper use in test env acquisition helpers |
| Functional | CLI contract tests that mutate signer env |
| Conformance | no direct poison-propagating signer env lock acquisition patterns remain |
| Regression | intentional lock poisoning with subsequent successful guard acquisition |
