# AGENTS.md
> **Contract**: Spec-first, issue-driven, TDD workflow enforced via 7 gated phases.
> Every change traces to a GitHub Issue. No phase may be skipped. Tests and CI are the authority — if this file conflicts with passing tests, update this file.
---
## §0 NON-NEGOTIABLES
1. **Issue-first.** No code without a GitHub Issue. Create one with `gh issue create` if none exists.
2. **Spec-before-code.** Write `specs/{issue}-{slug}.md` before any implementation.
3. **TDD mandatory.** Red → Green → Refactor. Refactor is NOT optional.
4. **Integration mandatory.** No standalone, mock-only, or unwired code ships.
5. **Hard-fail errors.** Raise errors. Never swallow. No silent fallbacks.
6. **Small & idempotent.** Small files, single-purpose functions, safe to re-run.
---
## §1 THE 7-PHASE GATE
**Announce each phase transition explicitly.** ("ENTERING PHASE 3: RED TESTS")
### Phase 1 — ISSUE
- Read issue fully: `gh issue view <id> --comments`
- Issue MUST contain: problem statement, acceptance criteria, non-goals
- If acceptance criteria are missing, comment to propose them. Do not code.
- One issue = one concern. Split compound issues.
### Phase 2 — SPEC
- Create: `specs/{issue}-{slug}.md`
- Required sections: Objective | Inputs/Outputs | Boundaries/Non-goals | Failure modes | Acceptance criteria (testable booleans) | Files to touch | Error semantics | Test plan
- Update issue with spec link. Commit spec.
- **GATE: Spec committed before any implementation or test code.**
### Phase 3 — RED (Failing Tests)
- Write tests derived from spec acceptance criteria
- Tests MUST fail. If they pass, investigate — feature exists or test is wrong
- Test error paths for every failure mode in spec
- No mocks except at true external boundaries (network, filesystem, clock)
- Commit: `test({issue}): red tests for {slug}`
- **GATE: All tests committed and failing.**
### Phase 4 — GREEN (Minimal Implementation)
- Write minimum code to pass tests. Nothing extra.
- Follow §2 code standards strictly
- Run tests after each logical commit
- Commit: `feat({issue}): {what}`
- **GATE: All tests green.**
### Phase 5 — REFACTOR (Mandatory — Cannot Be Skipped)
Checklist — every item must be verified:
[ ] No function > 25 lines — extract
[ ] No file > 200 lines — split
[ ] Zero duplication (DRY pass)
[ ] All names self-documenting — rename anything unclear
[ ] Single responsibility per function and module
[ ] Error handling matches §3
[ ] Idempotency verified per §2
[ ] Dead code, TODOs, placeholders removed
[ ] Linter/formatter clean
[ ] Full test suite still green
- Commit: `refactor({issue}): {what improved}`
- **GATE: Checklist complete, tests green, lint clean.**
### Phase 6 — INTEGRATION WIRING
**The most critical phase. No floating code.**
[ ] New code imported/called from real entrypoints (routes, handlers, CLI, workers, DI container)
[ ] Config/env vars validated at startup — fail loud on missing
[ ] DB migrations exist and run
[ ] No TODO stubs, no mock adapters in production paths
[ ] At least one integration test exercises the real path
[ ] Smoke test: run the app and exercise the feature
- If ANYTHING is disconnected → return to Phase 4
- Commit: `integrate({issue}): wire {feature} into {system}`
- **GATE: Feature callable from real application and producing correct results.**
### Phase 7 — CLOSE & PR
- Update spec with any deviations
- Push branch, open PR: `gh pr create --title "[{issue}] {title}" --body "Closes #{issue}"`
- PR body includes: issue link, spec link, what/why summary, test evidence
- Close issue: `gh issue close {id}`
- PR checklist:
[ ] Spec current
[ ] Tests added
[ ] Refactor complete
[ ] Integration verified
[ ] CI green
- Merge PR
---
## §2 CODE STANDARDS
### Size & Structure
- **Files**: ≤200 LOC. Split at logical seams.
- **Functions**: ≤25 LOC. Extract helpers aggressively.
- **Nesting**: ≤2 levels. Use guard clauses and early returns.
- **Single purpose**: one file = one concept. One function = one operation.
### Idempotency
- Write operations MUST be safe to retry (upserts, conflict resolution, idempotency keys)
- No "create then hope" — enforce uniqueness constraints
- Side effects centralized and explicit — never in constructors or imports
- If true idempotency is impossible, document in spec: what duplicates, how to detect, how to reconcile
### Data & Dependencies
- Immutable by default. Never mutate arguments.
- Validate external input at boundaries. Never trust upstream data.
- Wrap external services behind interfaces. Inject dependencies.
- New dependencies require spec justification. Prefer stdlib.
---
## §3 ERROR HANDLING
### Hard Fail — Always
- **Raise errors. Never swallow.** No empty catches. No silent defaults.
- The caller MUST know when something failed.
- No fallbacks unless spec explicitly allows AND fallback is observable (logged/metricked).
### Structured Errors
Every error includes:
- `code`: Machine-readable constant (`USER_NOT_FOUND`, `CONFIG_INVALID`)
- `message`: Human-readable description
- `context`: Debug payload (IDs, parameters, correlation IDs)
- `cause`: Wrapped underlying error (preserve stack traces)
### Boundary Rule
- **Interior code**: throw/return typed errors. Do not log.
- **Entrypoints** (handlers, CLI, jobs): catch, log once with correlation ID, translate to response.
- Use `Result<T, E>` / explicit return types for expected failure paths. Reserve exceptions for unexpected failures.
### Fail Fast
- Validate inputs at function entry. Return/throw immediately.
- Fail at startup for missing config/env — not at first request.
---
## §4 GIT DISCIPLINE
- **Branch**: `{issue}-{slug}`
- **Commits**: `{type}({issue}): {imperative description}`
- Types: `test`, `feat`, `fix`, `refactor`, `integrate`, `docs`, `chore`
- Commit after every meaningful change — commits are save points
- Preserve TDD commit arc (red → green → refactor → integrate). No squash.
- Never commit: secrets, .env, node_modules, build artifacts
---
## §5 BOUNDARIES
### ✅ ALWAYS
- Create issue before code
- Write spec before tests
- Write tests before implementation
- Refactor after implementation
- Verify integration wiring
- Run tests before every commit
### ⚠️ ASK FIRST
- Adding dependencies
- Modifying DB schemas
- Changing public API contracts
- Modifying CI/CD config
- Deleting files
### 🚫 NEVER
- Commit secrets or credentials
- Skip refactor phase
- Leave TODO/FIXME in merged code
- Ship unwired/standalone code
- Use mocks in integration tests
- Swallow errors silently
- Push directly to main
- Implement without a spec
- Delete or weaken tests to pass CI

## Shell-Surface DoR Gate

When work touches shell/python/workflow/template surface (`scripts/**`, `.github/workflows/**`,
`.github/ISSUE_TEMPLATE/**`, `.github/pull_request_template.md`), the issue body must include:

shell_loc_delta_estimate: <integer|0>
rust_loc_delta_estimate: <integer|0>
shell_to_rust_ratio_delta_estimate: <float|0.0>
shell_surface_mitigation_issue: <issue-id|None>

## Shell-Surface DoD Gate

When shell/python/workflow/template surface changed, closure comments and PR summaries must include:

shell_loc_delta_actual: <integer|0>
rust_loc_delta_actual: <integer|0>
shell_to_rust_ratio_delta_actual: <float|0.0>
shell_surface_ratio_target_status: improved|neutral|regressed_with_waiver
---
## §6 COMMANDS
> Adapt to your repo. Discover via package.json, Makefile, Cargo.toml, etc.
bash
# Targeted (prefer these — fast feedback)
FORMAT_FILE="<cmd> path/to/file"
LINT_FILE="<cmd> path/to/file"
TYPECHECK_FILE="<cmd> path/to/file"
TEST_FILE="<cmd> path/to/test"
# Full suite (run before PR)
FORMAT_ALL="<cmd>"
LINT_ALL="<cmd>"
BUILD="<cmd>"
TEST_ALL="<cmd>"

a
## §7 WHEN UNCERTAIN — STOP
> If requirements, behavior, integration points, or error semantics are unclear:
- Re-read the issue and spec and research
- Propose clarifying questions or spec amendments
- Do NOT guess. Do NOT write speculative code.
