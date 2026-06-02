/* eslint-disable camelcase */
exports.up = (pgm) => {
  pgm.createTable('streams', {
    id: { type: 'bigserial', primaryKey: true },
    stream_id: { type: 'bigint', notNull: true, unique: true },
    employer: { type: 'varchar(56)', notNull: true },
    employee: { type: 'varchar(56)', notNull: true },
    token: { type: 'varchar(56)', notNull: true },
    deposit: { type: 'numeric(39,0)', notNull: true },
    withdrawn: { type: 'numeric(39,0)', notNull: true, default: 0 },
    rate_per_second: { type: 'numeric(39,0)', notNull: true },
    start_time: { type: 'bigint', notNull: true },
    stop_time: { type: 'bigint', notNull: true, default: 0 },
    last_withdraw_time: { type: 'bigint', notNull: true },
    status: {
      type: 'varchar(16)',
      notNull: true,
      default: 'Active',
      check: "status IN ('Active','Paused','Cancelled','Exhausted')",
    },
    created_at: { type: 'timestamptz', notNull: true, default: pgm.func('now()') },
    updated_at: { type: 'timestamptz', notNull: true, default: pgm.func('now()') },
  });

  pgm.createIndex('streams', 'employer');
  pgm.createIndex('streams', 'employee');
  pgm.createIndex('streams', 'status');
};

exports.down = (pgm) => {
  pgm.dropTable('streams');
};
