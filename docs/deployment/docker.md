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
