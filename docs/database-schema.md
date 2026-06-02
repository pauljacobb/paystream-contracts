# Database Schema

PostgreSQL schema for indexing PayStream on-chain data, events, and user preferences.

## Migration Tool

[node-pg-migrate](https://github.com/salsita/node-pg-migrate) — migrations live in `db/migrations/`.

```bash
# Install
npm install node-pg-migrate pg

# Run migrations
DATABASE_URL=postgres://... npx node-pg-migrate up

# Rollback last migration
DATABASE_URL=postgres://... npx node-pg-migrate down

# Seed development data
DATABASE_URL=postgres://... node db/seeds/dev.js
```

Add to `package.json` scripts:
```json
"migrate": "node-pg-migrate up",
"migrate:down": "node-pg-migrate down",
"seed": "node db/seeds/dev.js"
```

---

## Tables

### `streams`

Indexed copy of on-chain stream state.

| Column | Type | Description |
|---|---|---|
| `id` | `bigserial` | Internal PK |
| `stream_id` | `bigint` | On-chain stream ID (unique) |
| `employer` | `varchar(56)` | Stellar address of employer |
| `employee` | `varchar(56)` | Stellar address of employee |
| `token` | `varchar(56)` | SEP-41 token contract address |
| `deposit` | `numeric(39,0)` | Total deposit (stroops) |
| `withdrawn` | `numeric(39,0)` | Total withdrawn (stroops) |
| `rate_per_second` | `numeric(39,0)` | Streaming rate (stroops/s) |
| `start_time` | `bigint` | Unix timestamp stream started |
| `stop_time` | `bigint` | Unix timestamp hard stop (0 = none) |
| `last_withdraw_time` | `bigint` | Unix timestamp of last withdrawal |
| `status` | `varchar(16)` | `Active`, `Paused`, `Cancelled`, `Exhausted` |
| `created_at` | `timestamptz` | Row creation time |
| `updated_at` | `timestamptz` | Row last updated time |

Indexes: `employer`, `employee`, `status`

---

### `events`

Immutable log of on-chain events emitted by the stream contract.

| Column | Type | Description |
|---|---|---|
| `id` | `bigserial` | Internal PK |
| `stream_id` | `bigint` | FK → `streams.stream_id` (nullable) |
| `event_type` | `varchar(32)` | e.g. `stream_created`, `withdraw`, `cancelled` |
| `ledger` | `bigint` | Stellar ledger sequence number |
| `timestamp` | `bigint` | Unix timestamp from ledger |
| `tx_hash` | `varchar(64)` | Transaction hash |
| `payload` | `jsonb` | Event-specific data |
| `created_at` | `timestamptz` | Row creation time |

Indexes: `stream_id`, `event_type`, `ledger`

---

### `users`

User preferences and profile data.

| Column | Type | Description |
|---|---|---|
| `id` | `bigserial` | Internal PK |
| `stellar_address` | `varchar(56)` | Stellar public key (unique) |
| `display_name` | `varchar(128)` | Optional display name |
| `email` | `varchar(256)` | Optional email for notifications |
| `role` | `varchar(16)` | `employer`, `employee`, or `admin` |
| `created_at` | `timestamptz` | Row creation time |
| `updated_at` | `timestamptz` | Row last updated time |

Indexes: `stellar_address` (unique)

---

### `notifications`

Pending and delivered notifications for users.

| Column | Type | Description |
|---|---|---|
| `id` | `bigserial` | Internal PK |
| `user_id` | `bigint` | FK → `users.id` |
| `stream_id` | `bigint` | FK → `streams.stream_id` (nullable) |
| `type` | `varchar(32)` | e.g. `near_exhaustion`, `stream_cancelled` |
| `message` | `text` | Human-readable notification text |
| `read` | `boolean` | Whether the user has read it |
| `created_at` | `timestamptz` | Row creation time |

Indexes: `user_id`, `(user_id, read)`

---

## Migration Files

| File | Description |
|---|---|
| `001_create_streams.js` | Create `streams` table |
| `002_create_events.js` | Create `events` table |
| `003_create_users.js` | Create `users` table |
| `004_create_notifications.js` | Create `notifications` table |
