# Issue #5336 Tasks

## Ordered Tasks
- [x] T1 (Regression): add failing lock-domain alias regression test in managed-backend test module.
- [x] T2 (Implementation): switch managed-backend env lock to shared signer test lock.
- [x] T3 (Conformance): run targeted signer parallel regression commands and confirm deterministic pass.
- [x] T4 (Structural): create docs-contract wave-4 harness and migrate 11 low-coupling suites.
- [x] T5 (Conformance): verify include_str count reduction and run wave-4 harness tests.
- [x] T6 (Governance): capture branch cleanup evidence and update R45 review status/priority markers.
- [x] T7 (Quality): run fmt, targeted clippy, and touched test suites.

## Dependency Notes
- T2 depends on T1 red evidence.
- T5 depends on T4.
- T7 runs after all implementation tasks.
