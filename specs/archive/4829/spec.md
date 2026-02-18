# Spec — Issue #4829

- Title: Subtask: create `lane_registry` source and manifest/symlink generation tooling
- Parent: Parent task `#4816`
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Create a deterministic lane registry source artifact and generator that validates and renders manifest/symlink lane artifacts from that source.

## Problem Statement

Manifests and wrapper symlinks currently live as static artifacts without a first-class registry source and generator contract, making drift detection and future generation workflows harder to enforce.

## Scope

In scope:
- add `scripts/framework/lane_registry.json` as source-of-truth registry
- add `scripts/framework/generate_lane_artifacts.py` with `check` and `render` modes
- add deterministic contract test for registry/generator behavior
- document lane registry generation contract

Out of scope:
- full retirement of static/manual manifest maintenance path (handled in `#4830`)
- broad lane behavioral changes unrelated to artifact generation

## Acceptance Criteria

- AC-1: Registry source exists and includes all current manifest entries plus wrapper wiring metadata required for generated artifacts.
- AC-2: Generator validates repository artifacts against registry (`check`) and can render equivalent artifacts to an isolated output root (`render`).
- AC-3: Deterministic tests enforce generator contract behavior and remain green under framework + CI regression suites.

## Conformance Cases

- C-01 (AC-1, Conformance): `scripts/framework/lane_registry.json` contains `schema_version=kamn.framework.lane-registry.v1`, `manifest_count=171`, `wrapper_count=112` with matching entry arrays.
- C-02 (AC-2, Functional): `bash scripts/framework/test_lane_registry_generation.sh` verifies check-mode markers and render-mode materialization of representative manifest/symlink artifacts.
- C-03 (AC-3, Integration/Regression): `bash scripts/framework/test_contract_framework.sh` and `bash scripts/ci/test_ci_tools.sh` pass with lane-registry generation guard included.

## Success Metrics / Signals

- Registry/generator contract can be executed deterministically from one command.
- Framework test entrypoint includes lane-registry generation validation.
- No drift reported by generator check-mode over repository artifacts.
