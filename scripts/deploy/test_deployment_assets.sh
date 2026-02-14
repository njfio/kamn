#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOCKERFILE="${DOCKERFILE_PATH:-$ROOT_DIR/Dockerfile}"
COMPOSE_FILE="${COMPOSE_FILE_PATH:-$ROOT_DIR/deploy/docker-compose.yml}"
K8S_MANIFEST="${K8S_MANIFEST_PATH:-$ROOT_DIR/deploy/k8s/kamn-node.yaml}"
DEPLOY_DOC="${DEPLOY_DOC_PATH:-$ROOT_DIR/docs/ops/deployment.md}"
DOCKERIGNORE_FILE="${DOCKERIGNORE_PATH:-$ROOT_DIR/.dockerignore}"

if [ ! -f "$DOCKERFILE" ]; then
  echo "expected Dockerfile for deployment assets story" >&2
  exit 1
fi
if [ ! -f "$COMPOSE_FILE" ]; then
  echo "expected docker-compose deployment file" >&2
  exit 1
fi
if [ ! -f "$K8S_MANIFEST" ]; then
  echo "expected kubernetes manifest for deployment assets story" >&2
  exit 1
fi
if [ ! -f "$DEPLOY_DOC" ]; then
  echo "expected deployment operations document" >&2
  exit 1
fi
if [ ! -f "$DOCKERIGNORE_FILE" ]; then
  echo "expected .dockerignore for deployment build-context hygiene" >&2
  exit 1
fi

if ! grep -q '^FROM rust:' "$DOCKERFILE"; then
  echo "expected Dockerfile multi-stage builder image marker" >&2
  exit 1
fi
if ! grep -q '^FROM debian:' "$DOCKERFILE"; then
  echo "expected Dockerfile runtime image marker" >&2
  exit 1
fi
if ! grep -q 'kamn-node' "$DOCKERFILE"; then
  echo "expected Dockerfile to build/copy kamn-node binary" >&2
  exit 1
fi

for required_path in ".git" "target" ".tmp"; do
  if ! grep -Eq "^${required_path}/?$" "$DOCKERIGNORE_FILE"; then
    echo "expected .dockerignore to exclude ${required_path}" >&2
    exit 1
  fi
done

if ! grep -q 'processor:' "$COMPOSE_FILE"; then
  echo "expected docker-compose processor service" >&2
  exit 1
fi
if ! grep -q 'listener:' "$COMPOSE_FILE"; then
  echo "expected docker-compose listener service" >&2
  exit 1
fi
if ! grep -q 'approver:' "$COMPOSE_FILE"; then
  echo "expected docker-compose approver service" >&2
  exit 1
fi
if ! grep -q -- '--runtime-mode' "$COMPOSE_FILE"; then
  echo "expected docker-compose runtime mode command marker" >&2
  exit 1
fi

restart_marker_count="$(grep -c 'restart: unless-stopped' "$COMPOSE_FILE" || true)"
if [ "$restart_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to include restart: unless-stopped markers" >&2
  exit 1
fi

if ! grep -q 'kind: Deployment' "$K8S_MANIFEST"; then
  echo "expected kubernetes deployment resources" >&2
  exit 1
fi
if ! grep -q 'name: kamn-processor' "$K8S_MANIFEST"; then
  echo "expected kubernetes processor deployment marker" >&2
  exit 1
fi
if ! grep -q 'name: kamn-listener' "$K8S_MANIFEST"; then
  echo "expected kubernetes listener deployment marker" >&2
  exit 1
fi
if ! grep -q 'name: kamn-approver' "$K8S_MANIFEST"; then
  echo "expected kubernetes approver deployment marker" >&2
  exit 1
fi
if ! grep -q 'KAMN_NODE_DAEMON_MAX_TICKS' "$K8S_MANIFEST"; then
  echo "expected kubernetes manifest daemon max-ticks env marker" >&2
  exit 1
fi
if ! grep -q 'KAMN_NODE_DAEMON_TICK_INTERVAL_MS' "$K8S_MANIFEST"; then
  echo "expected kubernetes manifest daemon tick-interval env marker" >&2
  exit 1
fi

if ! grep -q 'docker compose -f deploy/docker-compose.yml up' "$DEPLOY_DOC"; then
  echo "expected deployment doc compose command marker" >&2
  exit 1
fi
if ! grep -q 'kubectl apply -f deploy/k8s/kamn-node.yaml' "$DEPLOY_DOC"; then
  echo "expected deployment doc kubectl apply marker" >&2
  exit 1
fi
if ! grep -q '\.dockerignore' "$DEPLOY_DOC"; then
  echo "expected deployment doc to mention .dockerignore build-context hygiene" >&2
  exit 1
fi
if ! grep -q 'KAMN_NODE_DAEMON_MAX_TICKS' "$DEPLOY_DOC"; then
  echo "expected deployment doc daemon max-ticks env marker" >&2
  exit 1
fi
if ! grep -q 'KAMN_NODE_DAEMON_TICK_INTERVAL_MS' "$DEPLOY_DOC"; then
  echo "expected deployment doc daemon tick-interval env marker" >&2
  exit 1
fi

echo "deployment asset contract tests passed."
