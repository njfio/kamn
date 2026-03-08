# Objective
Map `LiveTransportKamnClient::submit_artifact()` onto the existing service content registration route so accepted live-transport tasks can submit artifacts over the real HTTP service path.

# Inputs/Outputs
- Input: `submit_artifact(task_id: &TaskId, artifact: Artifact)` on `LiveTransportKamnClient`
- Output: stable numeric `ArtifactId` alias for the service `content_id`

# Boundaries/Non-goals
- Do not add new service API routes.
- Do not add artifact retrieval or listing APIs.
- Do not change live `register()` or `search_agents()` support status.
- Keep task-acceptance preconditions fail-closed.

# Failure modes
- Unknown task aliases must return `SdkError::NotFound { entity: "task", ... }`.
- Unaccepted tasks must return `SdkError::Conflict("task must be accepted before artifact submission")`.
- Empty artifact name or bytes must return `SdkError::InvalidInput` on the same fields as the in-memory client.
- Empty or colliding service `content_id` values must fail closed.

# Acceptance criteria
- [ ] `submit_artifact()` sends `POST /v1/content/register` with `content:write` scope.
- [ ] The request payload contains the accepted service task id plus artifact name and bytes in deterministic JSON.
- [ ] Successful content registration returns a stable numeric `ArtifactId` alias derived from `content_id`.
- [ ] Unknown task aliases and unaccepted tasks fail closed before network emission.
- [ ] The live transport unsupported-method regression no longer lists `submit_artifact()`.

# Files to touch
- `crates/kamn-sdk/src/live/agent.rs`
- `crates/kamn-sdk/src/live/config.rs`
- `crates/kamn-sdk/src/live/state.rs`
- `crates/kamn-sdk/src/live/task_escrow.rs`
- `crates/kamn-sdk/tests/live_transport_task_escrow.rs`
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `specs/6588-map-live-transport-artifact-submission.md`
- `fixtures/ci/test_file_size_policy_baseline.env` only if inventory drift occurs

# Error semantics
- Preserve existing `SdkError` variants.
- Use `SdkError::InvalidInput` for artifact field validation.
- Use `SdkError::NotFound` for missing task aliases.
- Use `SdkError::Conflict` for unaccepted tasks or alias collisions.
- Use `SdkError::TransportFailure` for empty service identifiers.

# Test plan
- Extend the live transport contract test to include successful artifact submission over `POST /v1/content/register`.
- Add failure-path coverage for unknown task alias and unaccepted task submission.
- Update the unsupported-method regression to assert only the still-unmapped live methods remain unsupported.
- Run the full `kamn-sdk` test suite and strict clippy.
