#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_TOOLCHAIN_FILE="$ROOT_DIR/rust-toolchain.toml"
DOCKERFILE="$ROOT_DIR/Dockerfile"
SUPPLY_CHAIN_WORKFLOW="$ROOT_DIR/.github/workflows/ci-supply-chain-advisory.yml"
EXPECTED_TOOLCHAIN='channel = "1.88.1"'
EXPECTED_DOCKERFILE_BUILDER='FROM rust:1.88-bookworm AS builder'

if [ ! -f "$RUST_TOOLCHAIN_FILE" ]; then
  echo "expected rust-toolchain.toml to exist for advisory toolchain alignment" >&2
  exit 1
fi

if ! grep -Fq "$EXPECTED_TOOLCHAIN" "$RUST_TOOLCHAIN_FILE"; then
  echo "expected rust-toolchain.toml to pin advisory builder toolchain marker: $EXPECTED_TOOLCHAIN" >&2
  exit 1
fi

if ! grep -Fq "$EXPECTED_DOCKERFILE_BUILDER" "$DOCKERFILE"; then
  echo "expected Dockerfile builder to align with advisory Rust toolchain marker: $EXPECTED_DOCKERFILE_BUILDER" >&2
  exit 1
fi

if ! grep -Fq 'docker build -t kamn-supply-chain-advisory:${{ github.sha }} .' "$SUPPLY_CHAIN_WORKFLOW"; then
  echo "expected advisory workflow to build the root Dockerfile image" >&2
  exit 1
fi
