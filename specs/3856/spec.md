# Spec - Issue #3856

- Title: Subtask: assemble live validation artifact pack with deterministic schema and taxonomy markers
- Parent: #3855
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Release review requires complete, reproducible artifact packaging with deterministic schema and taxonomy markers.

## Objective

Deliver live-validation artifact-pack assembly with deterministic go/no-go schema and fail-closed policy validation coverage.

## Scope

In scope:
- Go/no-go evidence bundle generation with deterministic marker schema.
- Policy validation for GO/NO-GO and tampered artifact scenarios.
- Milestone-level aggregate lineage coverage within artifact generation tests.

Out of scope:
- CI workflow redesign.

## Acceptance Criteria

- AC-1: Artifact pack generation emits deterministic schema/marker fields.
- AC-2: Policy checker validates GO and NO-GO bundles deterministically.
- AC-3: Tampered/missing-marker bundles fail closed with deterministic diagnostics.

## Conformance Cases

- C-01 (AC-1/AC-2): `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` pass-path scenarios succeed.
- C-02 (AC-2): `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` NO-GO path validation succeeds deterministically.
- C-03 (AC-3): tampered decision and missing-evidence-marker fail-closed scenarios in `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` pass.

## Success Metrics

- Go/no-go bundle generation/policy contracts remain deterministic and fail closed on drift.
