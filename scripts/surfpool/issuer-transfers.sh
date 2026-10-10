#!/usr/bin/env bash
set -euo pipefail

RPC_URL="${RPC_URL:-http://127.0.0.1:8899}"
PAYER="${PAYER:-$HOME/.config/solana/id.json}"
STATE="${STATE:-scripts/state/surfpool-issuer.json}"
PHASE="${1:-initial}"

case "$PHASE" in
  initial|pre-record) ;;
  *)
    echo "usage: $0 [initial|pre-record]" >&2
    exit 2
    ;;
esac

exec cargo run -p issuer-scripts --locked -- \
  --rpc-url "$RPC_URL" \
  --payer "$PAYER" \
  transfers \
  --phase "$PHASE" \
  --state "$STATE"
