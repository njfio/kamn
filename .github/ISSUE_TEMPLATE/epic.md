---
name: Epic
about: Large cross-cutting initiative with child stories/tasks
title: "Epic: <domain> <goal>"
labels: ["type:epic", "process:issue-driven", "process:tdd", "status:todo"]
---

## Parent
- Program epic: #1

## Problem Statement
<Describe the problem and impact.>

## Scope Boundary
In scope:
- <item>

Out of scope:
- <item>

## Acceptance Criteria
1. <criterion>
2. <criterion>
3. <criterion>

## Required Test Categories
- Unit: <what this epic requires>
- Functional: <what this epic requires>
- Integration: <what this epic requires>
- Regression: <what this epic requires>
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
- <issue links>
