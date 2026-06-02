# ADR 0006: Use a CDN for PayStream frontend static assets

## Status

Proposed

## Context

The PayStream frontend is delivered as static assets (HTML/CSS/JS). These assets change infrequently compared to traffic volume.

We need a delivery strategy that provides:

- Fast global load times
- Reduced origin load (especially during traffic spikes)
- Predictable caching behavior
- Safe cache invalidation during releases

## Decision

Serve the built frontend static assets (e.g., Vite `dist/`) via a CDN with the following caching policy:

1. **Versioned immutable assets** (JS/CSS/images)
   - Use hashed filenames produced by the frontend build pipeline.
   - Configure CDN/edge cache headers:
     - `Cache-Control: public, max-age=31536000, immutable`
2. **HTML entrypoints** (`index.html` and other non-hashed HTML)
   - Use shorter TTL and revalidation:
     - `Cache-Control: public, max-age=60, must-revalidate`
3. **Cache invalidation strategy**
   - Prefer deploying new builds with new hashed asset names.
   - Invalidate only HTML (or purge all) as a fallback for emergency rollbacks.

## Rationale

- Immutable hashed assets allow long-lived caching without the risk of clients being stuck on old JS bundles.
- Short-lived HTML caching balances quick release adoption with reasonable performance.
- CDN reduces latency and cost by offloading static delivery from the origin.

## Consequences

- Release pipeline must upload the entire `dist/` output each time.
- Rollbacks should be performed by switching the CDN origin/version and purging HTML if needed.
- Observability should include CDN cache hit ratio and error rate.

