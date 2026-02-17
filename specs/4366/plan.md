# Plan: #4366 RED Deployment-Safety Evidence Gap Tests

## Approach

- Update `scripts/deploy/test_generate_gonogo_evidence_bundle.sh` milestone fixtures with explicit rotation taxonomy + boundary markers.
- Add targeted drift fixtures that mutate one marker cluster at a time and assert deterministic fail-closed reason codes.

## Risks

- Drift fixtures can become brittle if deterministic strings change.

## Mitigation

- Centralize expected marker strings in test blocks and keep assertions minimal but deterministic.
