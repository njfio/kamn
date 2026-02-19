# Milestone R27.6: Key Custody, Multi-Signer Controls, and Deployment Hardening

- Milestone: `R27.6 Key custody, multi-signer controls, and deployment hardening`
- GitHub milestone: `https://github.com/njfio/kamn/milestone/40`
- Status: Active

## Objective
Close production-readiness gaps around signer key custody by enforcing fail-closed key-source policy, deterministic reason taxonomy, and deployment hardening evidence.

## Scope
- Managed signer custody and provenance enforcement.
- Production fallback-key prohibition.
- Multi-signer and deployment readiness checks with deterministic evidence markers.

## Epic Chain
- `#3948` Epic: R27.6 harden key custody, multi-signer policy, and deployment readiness closure.

## Stories and Tasks
- `#3950` Story: harden deployment secret handling and operator rotation runbooks with live validation.
- `#3953` Task: enforce production fallback-key prohibition and signer provenance runtime gates.
- `#3959` Subtask: enforce production fallback-key denylist and fail-closed signer provenance taxonomy.

## Exit Criteria
- Production-targeted signer policy denies fallback secret sources with deterministic reason markers.
- Reason taxonomy is validated by Rust tests and docs contracts.
- CI and local targeted suites pass for unit, functional, integration, and regression coverage.
