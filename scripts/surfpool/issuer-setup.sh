#!/usr/bin/env bash
set -euo pipefail

RPC_URL="${RPC_URL:-http://127.0.0.1:8899}"
PAYER="${PAYER:-$HOME/.config/solana/id.json}"
STATE="${STATE:-scripts/state/surfpool-issuer.json}"
HOLDERS="${HOLDERS:-5}"

exec cargo run -p issuer-scripts --locked -- \
  --rpc-url "$RPC_URL" \
  --payer "$PAYER" \
  setup \
  --output "$STATE" \
  --holders "$HOLDERS"
