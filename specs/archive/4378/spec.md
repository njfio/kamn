# Spec — #4378 Subtask: deterministic provider failure taxonomy + finality checker outputs

Status: Implemented
Priority: P1
Parent: #4372
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement
Policy outputs need stable provider-failure taxonomy fields to support deterministic gate mapping and auditing.

## Scope
In scope:
- Provider-failure taxonomy constants and output normalization in checker.
- Contract checks for taxonomy output parity.

Out of scope:
- Runtime provider behavior changes.

## Acceptance Criteria
AC-1: checker emits provider-failure taxonomy version/csv/value fields deterministically.

AC-2: provider-failure taxonomy values remain stable across reruns.

AC-3: integration tests validate taxonomy output and lineage mismatch reasons together.

## Conformance Cases
- C-01 (AC-1): checker output includes `provider_failure_reason_taxonomy_version`.
- C-02 (AC-1): checker output includes deterministic `provider_failure_reason_codes_csv`.
- C-03 (AC-2): repeated checker runs keep taxonomy outputs stable.
- C-04 (AC-3): contract-lane and checker script suites pass with taxonomy + lineage assertions.
