# Contributing to KAMN

This file is the canonical contributor policy for this repository.
`AGENTS.md` is maintained as a compatibility redirect for agent tooling.

## Role

You are a disciplined software engineer working in this repository. You follow strict issue-driven development and test-driven development (TDD). All contributors and agents must follow this contract. You write production-grade code and leave an auditable trail of decisions and progress.

---

## 1) Work Intake Rules

Every change starts from a GitHub issue. No exceptions.

### Issue Hierarchy

```
Epic → Story → Task → Subtask
```

- Every Task and Subtask must point to exactly one parent issue.
- Do not start implementation work without a Task or Subtask issue.
- For broad work, create a parent epic and decompose into Stories, Tasks, and Subtasks.

### Issue Templates

All issues must be created from templates in `.github/ISSUE_TEMPLATE/`:
- `epic.md`
- `story.md`
- `task.md`
- `subtask.md`

Do not open ad-hoc issues for implementation work. If an issue is missing required sections, update it to match template structure before starting code changes.

### Labels (Required on Every Issue)

| Namespace  | Values |
|-----------|--------|
| Type      | `type:epic`, `type:story`, `type:task`, `type:subtask` |
| Area      | `area:program`, `area:foundation`, `area:identity`, `area:messaging`, `area:channels`, `area:crypto`, `area:reputation`, `area:tasks`, `area:economics`, `area:bridges`, `area:sdk`, `area:infra`, `area:security`, `area:governance`, `area:dashboard`, `area:ci`, `area:backend`, `area:frontend`, `area:networking`, `area:qa`, `area:devops`, `area:docs` |
| Process   | `process:issue-driven`, `process:tdd` |
| Priority  | `priority:P0`, `priority:P1`, `priority:P2` |
| Status    | `status:todo`, `status:in-progress`, `status:done` |

Do not invent new label namespaces during execution. If a label is missing, map to the closest approved label and document the rationale in the issue body.

---

## 2) Definition of Ready (DoR)

A Task is ready for implementation when:

- Parent issue and objective are clear.
- Acceptance criteria are testable and unambiguous.
- Dependencies are identified and linked.
- Risk level is stated: `low` / `med` / `high`.
- Required labels (type, area, process, priority) are applied.

## Shell-Surface DoR Gate

If the change touches shell/python/workflow/template surface (`scripts/**`, `.github/workflows/**`, `.github/ISSUE_TEMPLATE/**`, `.github/pull_request_template.md`), the issue body must include:

```
shell_loc_delta_estimate: <integer|0>
rust_loc_delta_estimate: <integer|0>
shell_to_rust_ratio_delta_estimate: <float|0.0>
shell_surface_mitigation_issue: <issue-id|None>
```

Use conservative estimates when exact values are not yet known.

---

## 3) TDD Execution Standard (Mandatory)

Every Task and Subtask must follow this loop — no exceptions:

### The TDD Cycle

```
1. 🔴 Red     — Write or extend a failing test first.
2. 🟢 Green   — Implement the minimum code to pass.
3. 🔵 Refactor — Improve structure while tests remain green.
4. 🔁 Regression — Run the full relevant suite to confirm nothing broke.
```

You must capture evidence of the Red → Green transition. Include the failing test output and the passing test output in the PR.

### Minimum Testing Expectations

For each completed issue, include or update tests across all applicable categories. If a category does not apply, you must document why — silence is not acceptable.

#### Unit Tests
- Test individual functions and methods in isolation.
- Mock or stub external dependencies.
- Cover both happy paths and error/edge cases.
- Every public function must have at least one unit test.

#### Functional Tests
- Validate feature-level behavior against the issue's acceptance criteria.
- Each acceptance criterion should map to at least one functional test.
- Test the feature as a caller would invoke it, end to end within the module.

#### Integration Tests
- Verify cross-module, cross-crate, or cross-service interactions.
- Test real I/O paths (filesystem, network, database) where applicable.
- Confirm components compose correctly under realistic conditions.

#### Regression Tests
- For every bug fix: write a failing test that reproduces the bug first, then fix to green.
- Regression tests must reference the originating issue ID: `// Regression: #<id>`
- For refactors: ensure existing tests pass without modification unless the API changed intentionally.

#### Performance / Load Tests (When Applicable)
- Required for networking, replication, data pipeline, or high-throughput hotspots.
- Document baseline metrics and acceptable thresholds in the issue.

### Test Naming Convention

```
#[test]
fn <module>_<behavior>_<condition>() { ... }
// Example: fn auth_rejects_expired_token()
```

### Coverage Expectations
- New code must not decrease overall test coverage.
- Critical paths (auth, data mutation, error handling) require exhaustive coverage.
- If something cannot be tested, document why in the PR and create a follow-up issue.

---

## 4) Issue-Driven Execution Cadence

For every Task / Subtask, follow this exact sequence:

1. **Confirm readiness** — parent links present, all required labels applied, status set to `status:todo`.
2. **Move to in-progress** — set `status:in-progress` before any code edits.
3. **Create branch** from `main`: `codex/issue-<id>-<short-slug>`
4. **Execute strict TDD:**
   - Write failing tests.
   - Capture 🔴 Red test output.
   - Implement minimal fix.
   - Capture 🟢 Green test output.
   - Refactor while green.
   - Run full regression suite.
5. **Update docs** — architecture, planning, or API docs as needed in the same PR.
6. **Open PR** with `Closes #<task>` (and `Closes #<subtask>` where applicable).
7. **Merge** only after all required checks pass.
8. **Verify closure** — confirm remote issue state is CLOSED and label progression is complete.

### Process Logging

Log each significant step as an issue comment:

```
**Status:** 🟡 In Progress | 🟢 Complete | 🔴 Blocked
**Step:** <what you did>
**Result:** <outcome or finding>
**Next:** <planned next action>
```

---

## 5) Branching and Commits

### Branch Strategy
- `main` is stable and releasable.
- All implementation work happens on feature branches.
- Branch naming: `codex/issue-<id>-<short-slug>`

### Commit Format (Conventional Commits + Issue Reference)

```
feat(auth): implement token refresh flow (#42)
test(parser): add edge-case coverage for empty input (#55)
docs(architecture): update data-flow diagram (#60)
chore(governance): add subtask template (#61)
```

### Commit Hygiene
- Keep commits atomic by concern: tests/fixtures, implementation, docs/governance.
- One logical change per commit.
- Rebase/sync frequently to reduce merge drift.

---

## 6) Pull Requests

Every PR must link its issue: `Closes #<id>` or `Refs #<id>`.

### PR Description Template

```
## Summary
<1-3 sentences: what changed and why>

## Changes
- <file or module>: <what changed>

## TDD Evidence (Mandatory)
### 🔴 Red Phase
<paste failing test command + output>

### 🟢 Green Phase
<paste passing test command + output>

### 🔁 Regression
<paste full suite command + pass/fail summary>

## Test Matrix (Mandatory)
For each category, provide status and specifics. If N/A, state why.

| Category      | Status     | Tests Added/Modified         | Justification if N/A |
|--------------|------------|------------------------------|----------------------|
| Unit         | ✅/❌/N/A |                              |                      |
| Functional   | ✅/❌/N/A |                              |                      |
| Integration  | ✅/❌/N/A |                              |                      |
| Regression   | ✅/❌/N/A |                              |                      |
| Performance  | ✅/❌/N/A |                              |                      |

## Risks & Rollback
- <breaking changes, deprecations, migration notes — write "None" if clean>
- <rollback plan if this change causes production issues>

## Documentation Updates
- <list files updated, or "None required" with justification>

## Validation Checklist
- [ ] `cargo fmt --check` — clean
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] TDD Red/Green evidence included above
- [ ] Full regression suite passing
- [ ] Docs updated (or justified why not)
```

### PR Merge Blockers
- Incomplete test matrix (blank rows or unjustified N/A).
- Missing TDD Red/Green evidence.
- Failing CI checks.
- No linked issue.
- Significant behavior changes without docs updates.

---

## 7) Definition of Done (DoD)

A Task is done only when:

- All acceptance criteria are satisfied.
- Tests were written first (TDD) and are passing.
- TDD Red → Green evidence is captured in the PR.
- Full regression suite passes with no new failures.
- Relevant documentation was updated.
- PR references the issue and includes complete test matrix.
- No critical lint/type/test regressions remain.
- Issue is closed with a closing comment:

```
**Outcome:** <what was delivered>
**PR:** #<number>
**Test Summary:** <tests added/modified by category>
**Follow-up:** <remaining work, or "None">
```

## Shell-Surface DoD Gate

When shell/python/workflow/template surface changed, PR summary and issue closure must include measured markers:

```
shell_loc_delta_actual: <integer|0>
rust_loc_delta_actual: <integer|0>
shell_to_rust_ratio_delta_actual: <float|0.0>
shell_surface_ratio_target_status: improved|neutral|regressed_with_waiver
```

`regressed_with_waiver` requires a linked mitigation follow-up issue.

---

## 8) GitHub Issue Quality Bar

Every implementation issue must include:

- Problem statement and user/system impact.
- Explicit acceptance criteria with deterministic test signals.
- TDD checklist as checkboxes:
  - [ ] 🔴 Red — failing test written
  - [ ] 🟢 Green — minimal implementation passing
  - [ ] 🔵 Refactor — code improved, tests still green
  - [ ] 🔁 Regression — full suite clean
- Dependencies and blocking issues.
- Required documentation updates (specific file paths).
- Risk level: `low` / `med` / `high`.

---

## 9) Commands

```bash
# Format
cargo fmt --check

# Lint (strict)
cargo clippy -- -D warnings

# Test — single module
cargo test -p <crate> -- <test_name>

# Test — full suite
cargo test

# Build
cargo build --release
```

Always run fmt, clippy, and relevant tests before committing. Run the full suite before opening a PR.

---

## 10) Documentation Discipline

- Architecture contracts and design decisions → `docs/architecture/`
- Research outputs → `docs/research/`
- Backlog maps and delivery plans → `docs/planning/`
- Each completed Task must update at least one corresponding doc.
- Do not merge significant behavior changes without docs updates.

---

## 11) Subagent Research Pattern

When planning major features, split research into dedicated streams:

| Subagent | Focus Area |
|----------|-----------|
| A | Backend / service architecture |
| B | Networking / protocol / API design |
| C | Frontend / rendering / UX constraints |
| D | Test strategy and automation |
| E | Workflow and delivery governance |
| F | Legal / IP / compliance risk |

Record findings in `docs/research/` before implementation begins.

---

## 12) Safety and Permissions

**Allowed without prompt:**
- Read/list files and directories
- Run fmt, clippy, and test commands
- Create branches and commits

**Ask first:**
- Adding new dependencies
- Deleting files or modules
- Modifying CI/CD configuration
- Running full release builds
- Any operation touching secrets, credentials, or environment config

**Never:**
- Commit secrets, tokens, or API keys
- Force-push to `main` or protected branches
- Remove or skip failing tests to make CI pass
- Mark a test category as N/A without written justification

---

## 13) Roadmap Tracking

- Use milestones for multi-wave work.
- Roadmap issues must specify: problem statement, scope boundary, acceptance criteria, and required test matrix.
- Keep epics updated with child-issue status as work progresses.

---

## 14) When Stuck

- Ask a clarifying question or propose a short plan — do not push large speculative changes.
- If blocked, update the issue with a 🔴 Blocked status comment and tag the relevant person.
- Open a draft PR with `[WIP]` prefix if partial progress is worth preserving.

---

## Do

- Follow TDD religiously — no implementation code without a failing test first.
- Default to small, focused diffs.
- Prefer explicit error handling over silent failures.
- Write self-documenting code; add doc comments on public APIs.
- Keep functions short and single-purpose.
- Write tests before or alongside implementation, never as an afterthought.
- Capture Red/Green evidence for every change.

## Don't

- Don't write implementation code before a failing test exists.
- Don't introduce new dependencies without justification in the PR.
- Don't leave `unwrap()` in production paths — use proper error handling.
- Don't modify unrelated files in a PR scoped to a specific issue.
- Don't skip tests to meet a deadline.
- Don't mark a test category as N/A without written justification.
- Don't merge without TDD evidence in the PR.
- Don't invent new label namespaces without governance approval.
