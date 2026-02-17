#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOCKERFILE="${DOCKERFILE_PATH:-$ROOT_DIR/Dockerfile}"
COMPOSE_FILE="${COMPOSE_FILE_PATH:-$ROOT_DIR/deploy/docker-compose.yml}"
K8S_MANIFEST="${K8S_MANIFEST_PATH:-$ROOT_DIR/deploy/k8s/kamn-node.yaml}"
DEPLOY_DOC="${DEPLOY_DOC_PATH:-$ROOT_DIR/docs/ops/deployment.md}"
DOCKER_DEPLOY_DOC="${DOCKER_DEPLOY_DOC_PATH:-$ROOT_DIR/docs/deployment/docker.md}"
DOCKERIGNORE_FILE="${DOCKERIGNORE_PATH:-$ROOT_DIR/.dockerignore}"
TLS_CERTS_README="${TLS_CERTS_README_PATH:-$ROOT_DIR/deploy/certs/README.md}"

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
if [ ! -f "$DOCKER_DEPLOY_DOC" ]; then
  echo "expected docker deployment topology document" >&2
  exit 1
fi
if [ ! -f "$DOCKERIGNORE_FILE" ]; then
  echo "expected .dockerignore for deployment build-context hygiene" >&2
  exit 1
fi
if [ ! -f "$TLS_CERTS_README" ]; then
  echo "expected deploy/certs/README.md for compose tls material contract" >&2
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
full_mode_marker_count="$(grep -c '^[[:space:]]*-[[:space:]]*full$' "$COMPOSE_FILE" || true)"
if [ "$full_mode_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to run in runtime-mode full" >&2
  exit 1
fi
api_bind_marker_count="$(grep -c -- '--api-bind' "$COMPOSE_FILE" || true)"
if [ "$api_bind_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to declare --api-bind endpoints" >&2
  exit 1
fi
service_api_tls_mode_marker_count="$(grep -c 'KAMN_SERVICE_API_TLS_MODE=require' "$COMPOSE_FILE" || true)"
if [ "$service_api_tls_mode_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to require service api tls mode" >&2
  exit 1
fi
service_api_tls_cert_marker_count="$(grep -c 'KAMN_SERVICE_API_TLS_CERT_FILE=/tls/service-api-cert.pem' "$COMPOSE_FILE" || true)"
if [ "$service_api_tls_cert_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to declare service api tls cert path" >&2
  exit 1
fi
service_api_tls_key_marker_count="$(grep -c 'KAMN_SERVICE_API_TLS_KEY_FILE=/tls/service-api-key.pem' "$COMPOSE_FILE" || true)"
if [ "$service_api_tls_key_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to declare service api tls key path" >&2
  exit 1
fi
tls_volume_marker_count="$(grep -c './certs:/tls:ro' "$COMPOSE_FILE" || true)"
if [ "$tls_volume_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to mount tls material volume" >&2
  exit 1
fi
for required_port in '19081:19081' '19082:19082' '19083:19083'; do
  if ! grep -q "$required_port" "$COMPOSE_FILE"; then
    echo "expected docker-compose port mapping marker ${required_port}" >&2
    exit 1
  fi
done
for required_volume in 'processor_data:/data/processor' 'listener_data:/data/listener' 'approver_data:/data/approver'; do
  if ! grep -q "$required_volume" "$COMPOSE_FILE"; then
    echo "expected docker-compose named volume marker ${required_volume}" >&2
    exit 1
  fi
done
for required_volume_declaration in 'processor_data:' 'listener_data:' 'approver_data:'; do
  if ! grep -q "^[[:space:]]*${required_volume_declaration}" "$COMPOSE_FILE"; then
    echo "expected docker-compose volume declaration marker ${required_volume_declaration}" >&2
    exit 1
  fi
done
if ! grep -q '^networks:' "$COMPOSE_FILE"; then
  echo "expected docker-compose network declaration marker" >&2
  exit 1
fi
if ! grep -q '^  kamn_mesh:' "$COMPOSE_FILE"; then
  echo "expected docker-compose named network marker kamn_mesh" >&2
  exit 1
fi

restart_marker_count="$(grep -c 'restart: unless-stopped' "$COMPOSE_FILE" || true)"
if [ "$restart_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to include restart: unless-stopped markers" >&2
  exit 1
fi
healthcheck_marker_count="$(grep -c 'healthcheck:' "$COMPOSE_FILE" || true)"
if [ "$healthcheck_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to include healthcheck blocks" >&2
  exit 1
fi
healthz_probe_marker_count="$(grep -c '/healthz' "$COMPOSE_FILE" || true)"
if [ "$healthz_probe_marker_count" -lt 3 ]; then
  echo "expected docker-compose triad services to probe /healthz endpoints" >&2
  exit 1
fi
if ! grep -q 'curl --fail --silent --insecure https://127.0.0.1:19081/healthz > /dev/null' "$COMPOSE_FILE"; then
  echo "expected docker-compose processor healthcheck probe marker" >&2
  exit 1
fi
if ! grep -q 'curl --fail --silent --insecure https://127.0.0.1:19082/healthz > /dev/null' "$COMPOSE_FILE"; then
  echo "expected docker-compose listener healthcheck probe marker" >&2
  exit 1
fi
if ! grep -q 'curl --fail --silent --insecure https://127.0.0.1:19083/healthz > /dev/null' "$COMPOSE_FILE"; then
  echo "expected docker-compose approver healthcheck probe marker" >&2
  exit 1
fi
depends_on_health_marker_count="$(grep -c 'condition: service_healthy' "$COMPOSE_FILE" || true)"
if [ "$depends_on_health_marker_count" -lt 2 ]; then
  echo "expected docker-compose listener/approver dependencies to require service_healthy" >&2
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
if ! grep -q 'runtime-mode full' "$DEPLOY_DOC"; then
  echo "expected deployment doc runtime-mode full marker" >&2
  exit 1
fi
if ! grep -q 'healthcheck' "$DEPLOY_DOC"; then
  echo "expected deployment doc healthcheck marker" >&2
  exit 1
fi
if ! grep -q '/healthz' "$DEPLOY_DOC"; then
  echo "expected deployment doc /healthz marker" >&2
  exit 1
fi
if ! grep -q 'service_healthy' "$DEPLOY_DOC"; then
  echo "expected deployment doc service_healthy dependency marker" >&2
  exit 1
fi
if ! grep -q 'KAMN_SERVICE_API_TLS_MODE=require' "$DEPLOY_DOC"; then
  echo "expected deployment doc service api tls mode marker" >&2
  exit 1
fi
if ! grep -q '\./certs:/tls:ro' "$DEPLOY_DOC"; then
  echo "expected deployment doc tls material volume marker" >&2
  exit 1
fi
if ! grep -q 'https://127.0.0.1:19081/healthz' "$DEPLOY_DOC"; then
  echo "expected deployment doc https healthcheck marker" >&2
  exit 1
fi
if ! grep -q '19081:19081' "$DEPLOY_DOC"; then
  echo "expected deployment doc processor api port marker" >&2
  exit 1
fi
if ! grep -q 'kamn_mesh' "$DEPLOY_DOC"; then
  echo "expected deployment doc named network marker" >&2
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
if ! grep -q 'deploy/certs/README.md' "$DEPLOY_DOC"; then
  echo "expected deployment doc to mention deploy/certs/README.md tls setup marker" >&2
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
for required_marker in \
  'deploy/docker-compose.yml' \
  'runtime-mode full' \
  '19081:19081' \
  '19082:19082' \
  '19083:19083' \
  'processor_data' \
  'listener_data' \
  'approver_data' \
  'kamn_mesh' \
  'healthcheck' \
  '/healthz' \
  'service_healthy' \
  'KAMN_SERVICE_API_TLS_MODE=require' \
  './certs:/tls:ro' \
  'https://127.0.0.1:19081/healthz' \
  'deploy/certs/service-api-cert.pem' \
  'deploy/certs/service-api-key.pem'; do
  if ! grep -q "$required_marker" "$DOCKER_DEPLOY_DOC"; then
    echo "expected docker deployment doc marker ${required_marker}" >&2
    exit 1
  fi
done

echo "deployment asset contract tests passed."
