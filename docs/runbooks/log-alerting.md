# Log-Based Alerting Runbook

## Overview

The `log-alerting.yml` workflow runs every minute and fires alerts to **Slack** and **PagerDuty** when error thresholds are exceeded.

## Alert Rules

| Alert | Threshold | Severity |
|---|---|---|
| 5xx error rate | > 1% of calls in last minute | Critical |
| Auth failures | > 10 per minute | Error |
| Contract call failures | > 1 per minute | Warning |

## Required Secrets

| Secret | Description |
|---|---|
| `STREAM_CONTRACT_ID` | Deployed stream contract address |
| `HORIZON_URL` | Horizon API base URL (defaults to testnet) |
| `SLACK_WEBHOOK_URL` | Slack incoming webhook URL |
| `PAGERDUTY_ROUTING_KEY` | PagerDuty Events API v2 routing key |

## Alert Destinations

- **Slack**: Posts to the configured webhook channel with emoji severity indicators.
- **PagerDuty**: Creates an incident via the Events API v2 (`/v2/enqueue`).

## Silencing / Adjusting Thresholds

Edit the `env` block at the top of `.github/workflows/log-alerting.yml`:

```yaml
env:
  ERROR_RATE_5XX_THRESHOLD: "0.01"   # raise to 0.05 for 5%
  AUTH_FAILURE_THRESHOLD: "10"       # raise to 20 if noisy
  CONTRACT_CALL_FAILURE_THRESHOLD: "1"
```

## Manual Trigger

Go to **Actions → Log-Based Alerting → Run workflow** to run an immediate check outside the scheduled window.
