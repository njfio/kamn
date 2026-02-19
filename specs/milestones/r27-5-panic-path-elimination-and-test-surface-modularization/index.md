# R27.5 Panic-path elimination and test-surface modularization

Milestone scope for removing panic primitives from production startup/runtime paths and decomposing node test surfaces with deterministic governance checks.

## Linked Issues
- #3933 Epic: R27.5 remove panic paths and modularize node test surface for reliability closure
- #3934 Story: eliminate production panic paths with typed fail-closed error handling
- #3935 Story: decompose node test monoliths and enforce test-surface governance
- #3936 Task: replace production expect/unreachable call paths with typed errors
- #3941 Subtask: replace production unreachable!() branches with explicit typed errors
