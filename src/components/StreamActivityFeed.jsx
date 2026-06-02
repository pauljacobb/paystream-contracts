import { useState, useEffect, useCallback } from 'react';

const PAGE_SIZE = 10;
const REFRESH_INTERVAL = 30000;

async function fetchStreamEvents(streamId, page) {
  const rpcUrl = import.meta.env.VITE_SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org';
  const response = await fetch(rpcUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'getEvents',
      params: {
        startLedger: 0,
        filters: [{ type: 'contract', contractIds: [], topics: [[streamId]] }],
        pagination: { limit: PAGE_SIZE, cursor: String(page * PAGE_SIZE) },
      },
    }),
  });
  const json = await response.json();
  return json.result?.events ?? [];
}

export default function StreamActivityFeed({ streamId }) {
  const [events, setEvents] = useState([]);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const load = useCallback(async () => {
    if (!streamId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await fetchStreamEvents(streamId, page);
      setEvents(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, [streamId, page]);

  useEffect(() => {
    load();
    const timer = setInterval(load, REFRESH_INTERVAL);
    return () => clearInterval(timer);
  }, [load]);

  if (loading) return <p>Loading...</p>;
  if (error) return <p>Error: {error}</p>;
  if (events.length === 0) return <p>No activity yet for this stream.</p>;

  return (
    <div>
      <ul>
        {events.map((ev, i) => (
          <li key={ev.id ?? i}>
            <span>{ev.type ?? 'event'}</span>
            {' · '}
            <span>{ev.value?.amount ?? '—'}</span>
            {' · '}
            <span>{ev.ledgerClosedAt ?? '—'}</span>
          </li>
        ))}
      </ul>
      <div>
        <button onClick={() => setPage((p) => Math.max(0, p - 1))} disabled={page === 0}>
          Previous
        </button>
        <span>Page {page + 1}</span>
        <button onClick={() => setPage((p) => p + 1)} disabled={events.length < PAGE_SIZE}>
          Next
        </button>
      </div>
    </div>
  );
}
