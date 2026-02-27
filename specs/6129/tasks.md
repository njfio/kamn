# Tasks: Issue #6129

- T1 (Red): Add tests covering configurable constructor, zero-timeout rejection, and socket timeout application.
- T2 (Green): Implement timeout configuration plumbing from `ServiceApiClient` into `ServiceEndpoint::connect_tcp_stream`.
- T3 (Regression): Ensure existing `connect` call path remains default-compatible.
- T4 (Docs/Verify): Update SDK docs and run fmt/clippy/tests with conformance evidence.
