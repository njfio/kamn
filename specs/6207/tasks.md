# Tasks: Issue #6207 - AgentIdentity Secret Material Zeroization + Clone Removal

- [x] T1 (RED): add regression marker tests for clone-removal/zeroization contracts.
- [x] T2 (GREEN): remove `Clone` derive and implement `Drop` zeroization for key buffers.
- [x] T3 (REFACTOR): update tests relying on `AgentIdentity::clone()`.
- [x] T4 (VERIFY): run fmt, clippy, and `kamn-agent-lib` test suite.
