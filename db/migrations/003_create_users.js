/* eslint-disable camelcase */
exports.up = (pgm) => {
  pgm.createTable('users', {
    id: { type: 'bigserial', primaryKey: true },
    stellar_address: { type: 'varchar(56)', notNull: true, unique: true },
    display_name: { type: 'varchar(128)' },
    email: { type: 'varchar(256)' },
    role: {
      type: 'varchar(16)',
      notNull: true,
      default: 'employee',
      check: "role IN ('employer','employee','admin')",
    },
    created_at: { type: 'timestamptz', notNull: true, default: pgm.func('now()') },
    updated_at: { type: 'timestamptz', notNull: true, default: pgm.func('now()') },
  });

  pgm.createIndex('users', 'stellar_address', { unique: true });
};

exports.down = (pgm) => {
  pgm.dropTable('users');
};
