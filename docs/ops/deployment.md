# Deployment Assets (Phase 6.3)

This document defines the first production-service deployment asset slice for Story #2970 and Task #2971.

## Scope

- Multi-stage container build for `kamn-node`.
- Build-context hygiene via repository-level `.dockerignore`.
- Local multi-role topology via Docker Compose.
- Kubernetes manifest for processor/listener/approver role deployments.
- Low-cost contract checks for artifact integrity.

## Container Build

Build local image:

```bash
docker build -t kamn-node:local -f Dockerfile .
```

The image starts `kamn-node` in deterministic daemon mode by default and is overridden by compose to run each role in `runtime-mode full`.

Build-context hardening:

- `.dockerignore` excludes non-runtime paths (for example: `.git`, `target`, `.tmp`) to keep local image builds fast and cost-effective.

## Docker Compose Topology

Compose file: `deploy/docker-compose.yml`

- `processor`
- `listener`
- `approver`
- each service runs `runtime-mode full` with bounded long-lived daemon tick budgets.
- API ports are exposed for local process probing:
  - processor: `19081:19081`
  - listener: `19082:19082`
  - approver: `19083:19083`
- named volumes preserve per-role state:
  - `processor_data`
  - `listener_data`
  - `approver_data`
- TLS material volume is mounted read-only in each service: `./certs:/tls:ro`
- service API TLS env markers are configured per role:
  - `KAMN_SERVICE_API_TLS_MODE=require`
  - `KAMN_SERVICE_API_TLS_CERT_FILE=/tls/service-api-cert.pem`
  - `KAMN_SERVICE_API_TLS_KEY_FILE=/tls/service-api-key.pem`
- named bridge network: `kamn_mesh`
- each service defines a compose `healthcheck` probing local `/healthz` endpoints over HTTPS (`curl --insecure`), for example: `https://127.0.0.1:19081/healthz`.

Generate local cert/key material before `docker compose up` (or follow `deploy/certs/README.md`):

```bash
mkdir -p deploy/certs
openssl req -x509 -newkey rsa:2048 -nodes -days 365 \
  -keyout deploy/certs/service-api-key.pem \
  -out deploy/certs/service-api-cert.pem \
  -subj "/CN=localhost"
```
- listener/approver service dependencies require `service_healthy` on processor.
- each service includes `restart: unless-stopped` for resilient local process restarts.

Run all roles locally:

```bash
docker compose -f deploy/docker-compose.yml up
```

Stop and cleanup:

```bash
docker compose -f deploy/docker-compose.yml down
```

See detailed docker topology contract notes in `docs/deployment/docker.md`.

## Kubernetes Manifest

Manifest file: `deploy/k8s/kamn-node.yaml`

Includes:

- `Namespace` (`kamn-system`)
- `ConfigMap` for chain/runtime defaults
- env-driven daemon controls via `KAMN_NODE_DAEMON_MAX_TICKS` and `KAMN_NODE_DAEMON_TICK_INTERVAL_MS`
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
- `bash scripts/deploy/test_validate_deployment_assets_live.sh`
- `bash scripts/deploy/validate_deployment_assets_live.sh`
- `cargo fmt --check`

The deployment-assets contract test fails closed when any required artifact or required marker is missing.

## Live Validation Evidence

Task and subtask:

- Task: #2973
- Subtask: #2974

Deterministic success markers:

- `status=pass`
- `final_decision=GO`
- `asset_contract_status=verified`
- `fail_closed_status=verified`
- `compose_manifest_contract_status=verified`
- `compose_config_contract_status=verified`
- `k8s_manifest_contract_status=verified`

Compose topology packaging governance markers (Issue #4433):

- `packaging_reason_taxonomy_version=kamn.deploy.compose-packaging-reason-taxonomy.v1`
- `packaging_reason_codes_csv=compose_packaging_manifest_drift_detected,compose_packaging_config_drift_detected,compose_packaging_evidence_contract_drift_detected`
- `packaging_contract_evidence_status=verified`

Fail-closed markers:

- `bash scripts/deploy/validate_deployment_assets_live.sh --max-seconds nope`
- stderr marker: `max-seconds must be an integer`

- negative drill inside live lane:
  - invalid Dockerfile missing builder `FROM rust:` marker
  - deterministic reason marker: `expected Dockerfile multi-stage builder image marker`
  - invalid compose runtime-mode command marker drift
  - deterministic reason marker: `expected docker-compose runtime mode command marker`
  - invalid k8s daemon env marker drift
  - deterministic reason marker: `expected kubernetes manifest daemon max-ticks env marker`
