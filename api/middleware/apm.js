// SPDX-License-Identifier: Apache-2.0
'use strict';

/**
 * APM middleware — request latency tracking and error rate monitoring (#300).
 *
 * Tracks:
 *   - Request latency (p50 / p95 / p99) per route
 *   - Error rate (4xx / 5xx counts)
 *   - Alerts when p99 latency exceeds 2 000 ms
 *   - Trace ID linked to logs via existing correlationId header
 *
 * Exposes metrics at GET /metrics (JSON).
 */

const ALERT_P99_MS = parseInt(process.env.APM_ALERT_P99_MS, 10) || 2000;
const MAX_SAMPLES = parseInt(process.env.APM_MAX_SAMPLES, 10) || 1000;

/** Rolling window of latency samples per route label. */
const latencySamples = new Map(); // label → number[]

/** Request and error counters per route label. */
const counters = new Map(); // label → { requests, errors4xx, errors5xx }

function getOrCreate(map, key, factory) {
  if (!map.has(key)) map.set(key, factory());
  return map.get(key);
}

function percentile(sorted, p) {
  if (sorted.length === 0) return 0;
  const idx = Math.ceil((p / 100) * sorted.length) - 1;
  return sorted[Math.max(0, idx)];
}

function routeLabel(req) {
  // Use matched route pattern when available (e.g. /v1/api/streams/:id),
  // fall back to the raw path to avoid high-cardinality labels.
  const route = req.route ? req.route.path : req.path;
  return `${req.method} ${route}`;
}

/**
 * Express middleware: records latency and status for every request.
 * Attaches `req.apmStart` so downstream handlers can read elapsed time.
 */
function apmMiddleware(req, res, next) {
  const start = process.hrtime.bigint();
  req.apmStart = start;

  res.on('finish', () => {
    const elapsedMs = Number(process.hrtime.bigint() - start) / 1e6;
    const label = routeLabel(req);
    const status = res.statusCode;

    // Record latency sample (rolling window)
    const samples = getOrCreate(latencySamples, label, () => []);
    samples.push(elapsedMs);
    if (samples.length > MAX_SAMPLES) samples.shift();

    // Update counters
    const c = getOrCreate(counters, label, () => ({ requests: 0, errors4xx: 0, errors5xx: 0 }));
    c.requests += 1;
    if (status >= 500) c.errors5xx += 1;
    else if (status >= 400) c.errors4xx += 1;

    // Alert on p99 > threshold
    const sorted = [...samples].sort((a, b) => a - b);
    const p99 = percentile(sorted, 99);
    if (p99 > ALERT_P99_MS) {
      console.error(
        JSON.stringify({
          level: 'ALERT',
          event: 'p99_latency_exceeded',
          route: label,
          p99_ms: Math.round(p99),
          threshold_ms: ALERT_P99_MS,
          correlation_id: req.correlationId || null,
          timestamp: new Date().toISOString(),
        })
      );
    }

    // Structured trace log — links latency to correlation ID for log correlation
    if (process.env.NODE_ENV !== 'test') {
      console.log(
        JSON.stringify({
          level: 'TRACE',
          route: label,
          status,
          duration_ms: Math.round(elapsedMs),
          correlation_id: req.correlationId || null,
          timestamp: new Date().toISOString(),
        })
      );
    }
  });

  next();
}

/**
 * Returns a snapshot of current APM metrics.
 * Shape: { routes: { [label]: { p50, p95, p99, requests, errors4xx, errors5xx, error_rate } } }
 */
function getMetrics() {
  const routes = {};
  for (const [label, samples] of latencySamples.entries()) {
    const sorted = [...samples].sort((a, b) => a - b);
    const c = counters.get(label) || { requests: 0, errors4xx: 0, errors5xx: 0 };
    const totalErrors = c.errors4xx + c.errors5xx;
    routes[label] = {
      p50_ms: Math.round(percentile(sorted, 50)),
      p95_ms: Math.round(percentile(sorted, 95)),
      p99_ms: Math.round(percentile(sorted, 99)),
      sample_count: sorted.length,
      requests: c.requests,
      errors4xx: c.errors4xx,
      errors5xx: c.errors5xx,
      error_rate: c.requests > 0 ? (totalErrors / c.requests).toFixed(4) : '0.0000',
      alert_p99: percentile(sorted, 99) > ALERT_P99_MS,
    };
  }
  return {
    generated_at: new Date().toISOString(),
    alert_threshold_ms: ALERT_P99_MS,
    routes,
  };
}

/** Reset all metrics (useful in tests). */
function resetMetrics() {
  latencySamples.clear();
  counters.clear();
}

module.exports = { apmMiddleware, getMetrics, resetMetrics };
