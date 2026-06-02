/* eslint-disable camelcase */
exports.up = (pgm) => {
  pgm.createTable('notifications', {
    id: { type: 'bigserial', primaryKey: true },
    user_id: { type: 'bigint', notNull: true, references: '"users"(id)', onDelete: 'CASCADE' },
    stream_id: { type: 'bigint', references: '"streams"(stream_id)', onDelete: 'SET NULL' },
    type: { type: 'varchar(32)', notNull: true },
    message: { type: 'text', notNull: true },
    read: { type: 'boolean', notNull: true, default: false },
    created_at: { type: 'timestamptz', notNull: true, default: pgm.func('now()') },
  });

  pgm.createIndex('notifications', 'user_id');
  pgm.createIndex('notifications', ['user_id', 'read']);
};

exports.down = (pgm) => {
  pgm.dropTable('notifications');
};
