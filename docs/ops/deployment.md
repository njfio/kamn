# Deployment Assets (Phase 6.3)

This document defines the first production-service deployment asset slice for Story #2970 and Task #2971.

## Scope

- Multi-stage container build for `kamn-node`.
- Local multi-role topology via Docker Compose.
- Kubernetes manifest for processor/listener/approver role deployments.
- Low-cost contract checks for artifact integrity.

## Container Build

Build local image:

```bash
docker build -t kamn-node:local -f Dockerfile .
```

The image starts `kamn-node` in deterministic daemon mode by default and can be overridden per role at runtime.

## Docker Compose Topology

Compose file: `deploy/docker-compose.yml`

- `processor`
- `listener`
- `approver`

Run all roles locally:

```bash
docker compose -f deploy/docker-compose.yml up
```

Stop and cleanup:

```bash
docker compose -f deploy/docker-compose.yml down
```

## Kubernetes Manifest

Manifest file: `deploy/k8s/kamn-node.yaml`

Includes:

- `Namespace` (`kamn-system`)
- `ConfigMap` for chain/runtime defaults
- `Deployment` resources:
  - `kamn-processor`
  - `kamn-listener`
  - `kamn-approver`

Apply:

```bash
kubectl apply -f deploy/k8s/kamn-node.yaml
```

Delete:

```bash
kubectl delete -f deploy/k8s/kamn-node.yaml
```

## Validation

Low-cost local checks:

- `bash scripts/deploy/test_deployment_assets.sh`
- `cargo fmt --check`

The deployment-assets contract test fails closed when any required artifact or required marker is missing.
