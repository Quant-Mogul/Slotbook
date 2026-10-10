#!/usr/bin/env bash
set -euo pipefail

RPC_PORT="${RPC_PORT:-8899}"
WS_PORT="${WS_PORT:-8900}"

exec surfpool start \
  --network devnet \
  --no-tui \
  --no-studio \
  --no-deploy \
  --db :memory: \
  --port "$RPC_PORT" \
  --ws-port "$WS_PORT"
