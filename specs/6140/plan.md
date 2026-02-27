# Plan: Issue #6140

## Approach
1. Add a structural conformance test that currently fails (RED) by asserting helper-delegation markers absent/present in source.
2. Refactor `handle_service_api_http_route` into:
   - top-level method router
   - dedicated `dispatch_service_api_post_route` and `dispatch_service_api_get_route` helpers
3. Keep route and reason-code behavior unchanged.
4. Re-run targeted and broader service-api tests.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `specs/6140/spec.md`
- `specs/6140/plan.md`
- `specs/6140/tasks.md`

## Risks
- Behavior drift in status/reason-code mapping for route branches.
- Increased helper indirection reducing readability if over-split.
- Conflicts with auto-generated lifecycle artifacts from intake PRs.

## Mitigations
- Preserve existing branch bodies with minimal movement.
- Add structural and regression assertions.
- Run scoped service-api test suite after refactor.
- Rebase onto latest main and keep single source-of-truth lifecycle artifacts.

## Interfaces/Contracts
- No wire-format or API contract changes.
- Internal function boundaries change only.
