# Docker Topology Contract

This document defines the deterministic docker topology contract for long-lived local `kamn-node` operation.

## Compose File

Primary artifact: `deploy/docker-compose.yml`

Role services:

- `processor`
- `listener`
- `approver`

Each service is wired for `runtime-mode full` and includes:

- `--runtime-mode full`
- `--daemon-max-ticks 1000000`
- `--daemon-tick-interval-ms 250`
- `--api-bind 0.0.0.0:<role-port>`
- `KAMN_SERVICE_API_TLS_MODE=require`
- `KAMN_SERVICE_API_TLS_CERT_FILE=/tls/service-api-cert.pem`
- `KAMN_SERVICE_API_TLS_KEY_FILE=/tls/service-api-key.pem`
- `./certs:/tls:ro` volume mount for local TLS material

## Ports

Exposed API mappings:

- `19081:19081` (processor)
- `19082:19082` (listener)
- `19083:19083` (approver)

## Persistent Volumes

Named per-role volumes:

- `processor_data`
- `listener_data`
- `approver_data`

These are mounted to `/data/<role>` paths in each container so restarts preserve local state.

## Network

All services join the named bridge network `kamn_mesh` for deterministic local service discovery.

## Healthcheck And Restart

Each role service includes a compose `healthcheck` probing `/healthz` over HTTPS on its local API bind:

- processor: `curl --fail --silent --insecure https://127.0.0.1:19081/healthz`
- listener: `curl --fail --silent --insecure https://127.0.0.1:19082/healthz`
- approver: `curl --fail --silent --insecure https://127.0.0.1:19083/healthz`

`listener` and `approver` dependencies require `service_healthy` on `processor`, and all roles keep `restart: unless-stopped`.

## TLS Material

Compose mounts local TLS files from `deploy/certs` and requires:

- `deploy/certs/service-api-cert.pem`
- `deploy/certs/service-api-key.pem`

Generate local self-signed material:

```bash
mkdir -p deploy/certs
openssl req -x509 -newkey rsa:2048 -nodes -days 365 \
  -keyout deploy/certs/service-api-key.pem \
  -out deploy/certs/service-api-cert.pem \
  -subj "/CN=localhost"
```

## Commands

Start topology:

```bash
docker compose -f deploy/docker-compose.yml up
```

Stop topology:

```bash
docker compose -f deploy/docker-compose.yml down
```

## Contract Validation

Run deterministic topology contract checks:

```bash
bash scripts/deploy/test_deployment_assets.sh
bash scripts/deploy/validate_deployment_assets_live.sh
```
