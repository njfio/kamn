---
name: Story
about: User-facing slice of an epic
title: "Story: <user need> <outcome>"
labels: ["type:story", "process:issue-driven", "process:tdd", "status:todo"]
---

## Parent
- Program epic: #1
- Parent epic: #<epic-id>

## User Story
As a <role>, I need <capability> so that <value>.

## Problem Statement
<Describe the current gap and impact.>

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
