---
name: Task
about: Implementable task under a story
title: "Task: <specific implementation objective>"
labels: ["type:task", "process:issue-driven", "process:tdd", "status:todo"]
---

## Parent
- Parent story: #<story-id>

## Objective
<Concrete implementation objective.>

## Problem Statement
<Describe the gap this task resolves.>

## Scope Boundary
In scope:
- <item>

Out of scope:
- <item>

## Acceptance Criteria
1. <criterion>
2. <criterion>
3. <criterion>
4. Unit, Functional, Integration, and Regression tests are present and passing.

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
