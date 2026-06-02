// Seed data for development
// Run: node db/seeds/dev.js
// Requires DATABASE_URL env var

require('dotenv').config();
const { Pool } = require('pg');

const pool = new Pool({ connectionString: process.env.DATABASE_URL });

async function seed() {
  const client = await pool.connect();
  try {
    await client.query('BEGIN');

    // Users
    const { rows: users } = await client.query(`
      INSERT INTO users (stellar_address, display_name, email, role) VALUES
        ('GEMPLOYER1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'Acme Corp', 'payroll@acme.example', 'employer'),
        ('GEMPLOYEE1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'Alice Smith', 'alice@example.com', 'employee'),
        ('GEMPLOYEE2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'Bob Jones', 'bob@example.com', 'employee')
      ON CONFLICT (stellar_address) DO NOTHING
      RETURNING id, stellar_address
    `);
    console.log(`Seeded ${users.length} users`);

    // Streams
    const { rows: streams } = await client.query(`
      INSERT INTO streams (stream_id, employer, employee, token, deposit, rate_per_second, start_time, stop_time, last_withdraw_time, status) VALUES
        (1, 'GEMPLOYER1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'GEMPLOYEE1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5', 3600000000, 1000000, 1748000000, 0, 1748000000, 'Active'),
        (2, 'GEMPLOYER1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'GEMPLOYEE2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5', 7200000000, 2000000, 1748000000, 0, 1748000000, 'Active')
      ON CONFLICT (stream_id) DO NOTHING
      RETURNING stream_id
    `);
    console.log(`Seeded ${streams.length} streams`);

    // Events
    await client.query(`
      INSERT INTO events (stream_id, event_type, ledger, timestamp, payload) VALUES
        (1, 'stream_created', 1000000, 1748000000, '{"rate_per_second": 1000000}'),
        (2, 'stream_created', 1000001, 1748000001, '{"rate_per_second": 2000000}')
      ON CONFLICT DO NOTHING
    `);
    console.log('Seeded events');

    await client.query('COMMIT');
    console.log('Seed complete');
  } catch (err) {
    await client.query('ROLLBACK');
    console.error('Seed failed:', err.message);
    process.exit(1);
  } finally {
    client.release();
    await pool.end();
  }
}

seed();
