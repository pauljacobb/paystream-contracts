# Canary Deployment Runbook

## Overview

The canary deployment workflow routes **5% of traffic** to a new contract version before full rollout. If the error rate exceeds **1%** during the canary window, an automated rollback fires and a Slack alert is sent. After **1 hour** of healthy canary, the new version is promoted to 100%.

## Trigger

Go to **Actions → Canary Deployment → Run workflow** and supply:

| Input | Description |
|---|---|
| `new_wasm_hash` | WASM hash of the new contract version |
| `network` | `testnet` or `mainnet` |

## Rollout Stages

1. **5% canary** — `set_canary` routes 5% of calls to the new WASM hash.
2. **Monitor (1 hour)** — polls `canary_error_rate` every 60 s.
3. **Rollback** — if error rate > 1%, `rollback_canary` is called and Slack is notified.
4. **Full rollout** — if healthy for 1 hour, `promote_canary` upgrades all traffic.

## Required Secrets

| Secret | Description |
|---|---|
| `STELLAR_SECRET_KEY` | Admin keypair with upgrade authority |
| `STREAM_CONTRACT_ID` | Deployed stream contract address |
| `SLACK_WEBHOOK_URL` | Slack incoming webhook for alerts |

## Canary Metrics Dashboard

Monitor the canary via the Stellar Horizon API:

```bash
# Query recent contract events for error patterns
curl "https://horizon-testnet.stellar.org/contracts/$STREAM_CONTRACT_ID/events?limit=200"
```

Key metrics to watch:
- `canary_error_rate` — fraction of canary calls that returned an error
- `canary_call_count` — total calls routed to canary version
