# Tasks: Issue #6205 - Replace Pipe-Delimited Snapshot Journal Records with Serde JSON

- [x] T1 (RED): add helper-level tests for JSON line schema + malformed record rejection.
- [x] T2 (GREEN): switch snapshot journal append/parse implementation to serde JSON lines.
- [x] T3 (REFACTOR): update snapshot replay call sites for updated parse API shape.
- [x] T4 (VERIFY): run fmt/clippy and targeted snapshot persistence tests.
