CREATE TABLE IF NOT EXISTS block (
  height BIGINT PRIMARY KEY,
  transaction_count BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);
