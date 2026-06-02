const client = require('prom-client');

// Create a Registry which registers the metrics
const register = new client.Registry();

// Default metrics (cpu/memory etc.)
client.collectDefaultMetrics({ register });

// Histogram for compression ratio percent (0-100)
const compressionRatio = new client.Histogram({
  name: 'paystream_response_compression_ratio_percent',
  help: 'Response compression ratio in percent (higher is better)',
  labelNames: ['encoding', 'method', 'path'],
  buckets: [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100],
  registers: [register],
});

// Counter for number of compressed responses
const compressedResponses = new client.Counter({
  name: 'paystream_response_compressed_total',
  help: 'Total number of compressed responses',
  labelNames: ['encoding', 'method', 'path'],
  registers: [register],
});

function observeCompressionRatio(percent, labels = {}) {
  if (typeof percent !== 'number' || Number.isNaN(percent)) return;
  // clamp
  const value = Math.max(0, Math.min(100, percent));
  compressionRatio.observe(labels, value);
}

function incrementCompressed(labels = {}) {
  compressedResponses.inc(labels);
}

async function metricsEndpoint(req, res) {
  try {
    res.set('Content-Type', register.contentType);
    res.end(await register.metrics());
  } catch (err) {
    res.status(500).end(err.message);
  }
}

module.exports = {
  observeCompressionRatio,
  incrementCompressed,
  metricsEndpoint,
  register,
};
