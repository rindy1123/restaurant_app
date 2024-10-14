CREATE TABLE IF NOT EXISTS tables (
  id SERIAL PRIMARY KEY,
  table_number INT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);