# Issue 6226 Tasks

- T1 (Red/Conformance): Add/adjust SDK tests that assert invalid shared DID parse maps to `SdkError::InvalidInput`.
- T2 (Green/Implementation): Create `kamn-types` crate and wire workspace/dependencies.
- T3 (Green/Implementation): Migrate `kamn-sdk` `AgentDid` usage to shared `kamn-types::AgentDid`.
- T4 (Regression): Run targeted tests for `kamn-types`, `kamn-sdk`, and `kamn-agent-lib`.
- T5 (Verification): Confirm AC-to-test mapping and close issue with conformance summary.
