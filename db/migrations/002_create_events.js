/* eslint-disable camelcase */
exports.up = (pgm) => {
  pgm.createTable('events', {
    id: { type: 'bigserial', primaryKey: true },
    stream_id: { type: 'bigint', references: '"streams"(stream_id)', onDelete: 'CASCADE' },
    event_type: { type: 'varchar(32)', notNull: true },
    ledger: { type: 'bigint', notNull: true },
    timestamp: { type: 'bigint', notNull: true },
    tx_hash: { type: 'varchar(64)' },
    payload: { type: 'jsonb', default: '{}' },
    created_at: { type: 'timestamptz', notNull: true, default: pgm.func('now()') },
  });

  pgm.createIndex('events', 'stream_id');
  pgm.createIndex('events', 'event_type');
  pgm.createIndex('events', 'ledger');
};

exports.down = (pgm) => {
  pgm.dropTable('events');
};
