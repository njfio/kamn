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

## Shell-Surface Governance (Required when script/workflow/template surface changes)
- `shell_loc_delta_estimate`: <integer|0>
- `rust_loc_delta_estimate`: <integer|0>
- `shell_to_rust_ratio_delta_estimate`: <float|0.0>
- `shell_surface_mitigation_issue`: <issue-id|None>

## Dependencies
- #<issue-id>
