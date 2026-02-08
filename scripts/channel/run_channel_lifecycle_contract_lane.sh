#!/usr/bin/env bash
set -euo pipefail

cargo test -p kamn-core --lib channel_models::tests:: >/dev/null
cargo test -p kamn-core --test channel_models >/dev/null
cargo test -p kamn-core --test channel_models_docs >/dev/null

echo "channel lifecycle snapshot contract lane tests passed."
