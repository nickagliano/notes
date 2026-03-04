#!/usr/bin/env bash
set -euo pipefail

# notes/serve.sh — EPC service entry point
#
# EPC sets PORT to the value declared in eps.toml [service].port.
#
# To test manually:
#   ./serve.sh
#   open http://localhost:3001

cd "$(dirname "$0")"

cargo build --release --quiet
exec ./target/release/notes
