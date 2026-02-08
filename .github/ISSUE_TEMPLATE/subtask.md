---
name: Subtask
about: Smallest actionable unit under a task
title: "Subtask: <focused implementation step>"
labels: ["type:subtask", "process:issue-driven", "process:tdd", "status:todo"]
---

## Parent
- Parent task: #<task-id>

## Objective
<Concrete, narrow objective for this subtask.>

## Problem Statement
<Why this subtask is needed.>

## Scope Boundary
In scope:
- <item>

Out of scope:
- <item>

## Acceptance Criteria
1. <criterion>
2. <criterion>
3. Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Required Test Categories
- Unit: <tests>
- Functional: <tests>
- Integration: <tests>
- Regression: <tests>
- Performance: <required or justify N/A>

## Risk Level
`low` | `med` | `high`

## TDD Checklist
- [ ] 🔴 Red — failing test written
- [ ] 🟢 Green — minimal implementation passing
- [ ] 🔵 Refactor — code improved, tests still green
- [ ] 🔁 Regression — full suite clean

## Required Documentation Updates
- `docs/<path>`: <what must be updated>

## Dependencies
- #<issue-id>
