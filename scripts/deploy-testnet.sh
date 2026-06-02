#!/usr/bin/env bash
# Deploy PayStream contracts to Stellar testnet.
# Usage: ./scripts/deploy-testnet.sh
# Outputs TOKEN_CONTRACT_ID and STREAM_CONTRACT_ID to stdout and,
# when running inside GitHub Actions, to $GITHUB_OUTPUT.
set -euo pipefail

NETWORK="testnet"
SOURCE="${STELLAR_SOURCE_ACCOUNT:-default}"

echo "Deploying PayStream contracts to Stellar testnet..."

TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/paystream_token.wasm \
  --source "$SOURCE" --network "$NETWORK")
echo "Token:  $TOKEN_ID"

STREAM_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/paystream_stream.wasm \
  --source "$SOURCE" --network "$NETWORK")
echo "Stream: $STREAM_ID"

echo ""
echo "TOKEN_CONTRACT_ID=$TOKEN_ID"
echo "STREAM_CONTRACT_ID=$STREAM_ID"

# Emit GitHub Actions outputs when running in CI
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "token_id=$TOKEN_ID"  >> "$GITHUB_OUTPUT"
  echo "stream_id=$STREAM_ID" >> "$GITHUB_OUTPUT"
fi
