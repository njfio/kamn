#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOCKERFILE="$ROOT_DIR/Dockerfile"
COMPOSE_FILE="$ROOT_DIR/deploy/docker-compose.yml"
K8S_MANIFEST="$ROOT_DIR/deploy/k8s/kamn-node.yaml"
DEPLOY_DOC="$ROOT_DIR/docs/ops/deployment.md"

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

if ! grep -q 'docker compose -f deploy/docker-compose.yml up' "$DEPLOY_DOC"; then
  echo "expected deployment doc compose command marker" >&2
  exit 1
fi
if ! grep -q 'kubectl apply -f deploy/k8s/kamn-node.yaml' "$DEPLOY_DOC"; then
  echo "expected deployment doc kubectl apply marker" >&2
  exit 1
fi

echo "deployment asset contract tests passed."
